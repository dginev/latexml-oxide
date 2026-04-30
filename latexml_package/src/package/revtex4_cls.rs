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
    // Perl L29: `endfloats*` (starred) alongside `endfloats`.
    "endfloats*",
    // sub-styles
    "aps", "pra", "prb", "prc", "prd", "pre", "prl", "prstab", "rmp",
    "osa", "osameet", "opex", "tops", "josa",
  ].iter() {
    DeclareOption!(*option, None);
  }

  // Perl revtex4.cls.ltxml L41-45: amsfonts/amssymb/amsmath options push
  // the package into @revtex_toload; no-variants remove it. Packages are
  // NOT loaded until AFTER ProcessOptions + LoadClass + revtex4_support,
  // mirrored here via state flags.
  for pkg in ["amsfonts", "amssymb", "amsmath"].iter() {
    DeclareOption!(*pkg, {
      state::assign_value(&s!("revtex_load_{}", pkg), true, Some(Scope::Global));
    });
    let nopkg = s!("no{}", pkg);
    DeclareOption!(&nopkg, {
      state::assign_value(&s!("revtex_load_{}", pkg), false, Some(Scope::Global));
    });
  }

  // Perl L47-49: osajnl also pushes `graphics` onto @revtex_toload (deferred
  // load) and DefMacros \ocis -> \pacs. Defer graphics like the AMS bundle.
  DeclareOption!("osajnl", {
    state::assign_value("revtex_load_graphics", true, Some(Scope::Global));
    DefMacro!("\\ocis", "\\pacs");
  });

  // Anything else is for article
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;
  RequirePackage!("revtex4_support");
  // Perl L58: deferred RequirePackage of @revtex_toload. Apply tracked flags.
  for pkg in ["amsfonts", "amssymb", "amsmath"].iter() {
    if state::lookup_bool(&s!("revtex_load_{}", pkg)) {
      RequirePackage!(*pkg);
    }
  }
  if state::lookup_bool("revtex_load_graphics") {
    RequirePackage!("graphics");
  }
});
