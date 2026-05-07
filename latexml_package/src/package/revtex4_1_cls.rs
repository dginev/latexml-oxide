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
    "fleqn", "floats", "endfloats",
    // Perl L29 also declares the starred `endfloats*`; Rust was missing it
    // so `\documentclass[endfloats*]{revtex4-1}` fell through to the article
    // option passthrough.
    "endfloats*",
    "titlepage", "notitlepage",
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
  // Perl revtex4-1.cls.ltxml L41-45: amsfonts/amssymb/amsmath are in
  // @revtex_toload BY DEFAULT; positive options are no-ops, negative
  // (`noamsmath`/`noamssymb`/`noamsfonts`) options REMOVE from the list.
  // Was: defaulted to false and positive options set true — so
  // `\documentclass[amsmath,...]{revtex4-1}` (driver: 2210.07776) failed
  // to load amsmath because the DeclareOption handler appears not to fire
  // for already-positively-listed options under our ProcessOptions flow.
  // Mirror Perl's default-on behavior so `\boldsymbol` (defined in amsbsy
  // pulled by amsmath) is available throughout the doc.
  for pkg in ["amsfonts", "amssymb", "amsmath"].iter() {
    state::assign_value(&s!("revtex_load_{}", pkg), true, Some(Scope::Global));
    DeclareOption!(*pkg, None);
    let nopkg = s!("no{}", pkg);
    DeclareOption!(&nopkg, {
      state::assign_value(&s!("revtex_load_{}", pkg), false, Some(Scope::Global));
    });
  }
  // Perl L47-49: osajnl (Optical Society) sub-option also pushed graphics
  // onto @revtex_toload and Let-aliased \ocis → \pacs. Revtex4 class has
  // this (revtex4_cls.rs L37-39); the revtex4-1 port had dropped it,
  // which breaks OSA documents that rely on the \ocis classification.
  DeclareOption!("osajnl", {
    state::assign_value("revtex_load_graphics", true, Some(Scope::Global));
    DefMacro!("\\ocis", "\\pacs");
  });
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });
  ProcessOptions!();
  LoadClass!("article");
  RequirePackage!("revtex4_support");

  // Perl revtex4-1.cls.ltxml L60-63: auto-load `<jobname>.rty` if present.
  // Same convention as revtex4 — paper-local macros stashed in .rty file.
  Digest!("\\InputIfFileExists{\\jobname.rty}{}{}")?;
  // Load AMS packages that were requested via options
  for pkg in ["amsfonts", "amssymb", "amsmath"].iter() {
    if state::lookup_bool(&s!("revtex_load_{}", pkg)) {
      RequirePackage!(*pkg);
    }
  }
  if state::lookup_bool("revtex_load_graphics") {
    RequirePackage!("graphics");
  }
});
