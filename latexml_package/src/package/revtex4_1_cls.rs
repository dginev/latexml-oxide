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
  // Perl revtex4-1.cls.ltxml L40-45:
  //   my @revtex_toload = ();        # EMPTY by default
  //   foreach my $pkg (qw(amsfonts amssymb amsmath)) {
  //     DeclareOption($pkg,   sub { push(@revtex_toload, $pkg); });
  //     DeclareOption("no$pkg", sub { @revtex_toload = grep {…} … }); }
  // Same Perl-faithful empty default as revtex4_cls.rs sister fix.
  // Was: defaulted to TRUE with the misattributed claim that Perl's
  // default is the full list; Perl's actual default is `()`. The flip
  // pulled amsmath into papers that don't opt in, and amsmath's `\cases`
  // redefinition then mis-parsed plain TeX `\cases{… & … \cr}` inside
  // `\begin{equation}`. RUST REGRESSION — drop the default-true.
  //
  // Driver 2210.07776's `\boldsymbol undefined` cascade was the
  // originally-claimed motivation; if it regresses, the proper fix is
  // in the DeclareOption / ProcessOptions handler (separate work).
  for pkg in ["amsfonts", "amssymb", "amsmath"].iter() {
    let pkg_owned = pkg.to_string();
    DeclareOption!(*pkg, {
      state::assign_value(&s!("revtex_load_{}", pkg_owned), true, Some(Scope::Global));
    });
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

  // revtex4-1.cls L?: \providecommand\doi[0]{...} — used in bibliography
  // entries (e.g. `\doi{10.1103/PhysRevLett.123.210602}`). Real cls wraps
  // the doi in \@doi href but for XML we just emit the DOI as a hyperlink.
  // Witness 2403.08476. Use Semiverbatim so `_`, `&`, `#` in DOIs (e.g.
  // `\doi{10.1007/978-3-662-43948-7_26}` from 2112.03925) don't trigger
  // catcode errors (`Error:unexpected:_ Script _ can only appear in math
  // mode`) when emitted into the href text.
  DefMacro!("\\doi Semiverbatim", "doi:\\href{https://doi.org/#1}{#1}");
  DefMacro!("\\doibase",   "https://doi.org/");
});
