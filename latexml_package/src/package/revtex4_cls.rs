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

  // Perl revtex4.cls.ltxml L40-45:
  //   my @revtex_toload = ();        # EMPTY by default
  //   foreach my $pkg (qw(amsfonts amssymb amsmath)) {
  //     DeclareOption($pkg,   sub { push(@revtex_toload, $pkg); });
  //     DeclareOption("no$pkg", sub { @revtex_toload = grep {…} … }); }
  // i.e. amsfonts/amssymb/amsmath are only loaded if the user explicitly
  // passes that option to `\documentclass`. The earlier Rust port flipped
  // the default to TRUE — its comment claimed to mirror Perl's
  // `@revtex_toload = (amsfonts,amssymb,amsmath)` default, but Perl's
  // actual literal is `()`. The flip caused papers that don't opt in to
  // amsmath to nevertheless get amsmath's `\cases` redefinition, which
  // then mis-parses plain TeX `\cases{X & Y \cr}` and cascades into
  // `unexpected:\end{equation}` + downstream `unexpected:_/^`. RUST
  // REGRESSION — Perl-faithful fix: empty default, positive option ⇒
  // set load=true.
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

  // Perl revtex4.cls.ltxml L60-62: auto-load `<jobname>.rty` if present.
  // Papers like cond-mat0201306 stash paper-local macros (`\TR`, `\GC`,
  // `\bracketOpen` etc.) in this file via revtex's runtime convention.
  Digest!("\\InputIfFileExists{\\jobname.rty}{}{}")?;
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
