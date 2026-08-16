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
      assign_value(&s!("revtex_load_{}", pkg_owned), true, Some(Scope::Global));
    });
    let nopkg = s!("no{}", pkg);
    DeclareOption!(&nopkg, {
      assign_value(&s!("revtex_load_{}", pkg), false, Some(Scope::Global));
    });
  }

  // Perl L47-49: osajnl also pushes `graphics` onto @revtex_toload (deferred
  // load) and DefMacros \ocis -> \pacs. Defer graphics like the AMS bundle.
  DeclareOption!("osajnl", {
    assign_value("revtex_load_graphics", true, Some(Scope::Global));
    DefMacro!("\\ocis", "\\pacs");
  });

  // Anything else is for article
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  // REVTeX4-1/4-2 declare `author-year` / `numerical` and DEFAULT to numerical
  // (revtex4-2.cls L6022-6024, revtex4-1.cls L6001-6003 → `\@booleanfalse
  // \authoryear@sw`), i.e. bracketed numeric citations like the PDF. (Bare
  // revtex4 4.0 has no such toggle and is numeric too.) Neither Perl's revtex4*
  // bindings nor natbib's own default honor these class options, so oxide
  // SURPASSES Perl here to match the real class + PDF. The chosen style is
  // queued onto natbib below; a later `\setcitestyle` / `\bibpunct` still
  // overrides it, exactly as in Perl. html_feedback #6609, witness arXiv 2606.09494.
  DeclareOption!("numerical", {
    assign_value("revtex_cite_style", pin("numbers"), Some(Scope::Global));
  });
  DeclareOption!("author-year", {
    assign_value("revtex_cite_style", pin("authoryear"), Some(Scope::Global));
  });
  assign_value("revtex_cite_style", pin("numbers"), Some(Scope::Global));
  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;
  // Queue the citation style onto natbib rather than pre-loading natbib here.
  // `numbers` gives the numeric `[N]` refnum + square/comma inline; `author-year`
  // leaves natbib's own author-year (round/semicolon) default. `\PassOptionsToPackage`
  // is used ON PURPOSE instead of a direct `RequirePackage("natbib", [numbers])`:
  // pre-loading natbib here loads it BEFORE revtex4_support's hyperref, and that
  // load-order flip perturbs allocation enough to trip a latent libxml heisenbug
  // (intermittent SIGSEGV in the multiline_class_options structure test, whose
  // output is unchanged). Queuing keeps natbib in its original position inside
  // revtex4_support (after hyperref), so only the option changes. html_feedback #6609.
  if lookup_string("revtex_cite_style") == "numbers" {
    Digest!("\\PassOptionsToPackage{numbers}{natbib}")?;
  }
  RequirePackage!("revtex4_support");

  // Perl L58: deferred RequirePackage of @revtex_toload. Apply tracked flags.
  // Load AMS packages BEFORE the `.rty` input — Perl revtex4.cls.ltxml runs
  // `map { RequirePackage($_) } @revtex_toload` (L58) before the L61-62
  // `\jobname.rty` load, so a paper-local `.rty` using an AMS macro (e.g.
  // `\DeclareMathOperator`) finds it defined. Faithful order (sister fix to
  // revtex4_1_cls.rs, witness 1508.02642).
  for pkg in ["amsfonts", "amssymb", "amsmath"].iter() {
    if lookup_bool(&s!("revtex_load_{}", pkg)) {
      RequirePackage!(*pkg);
    }
  }
  if lookup_bool("revtex_load_graphics") {
    RequirePackage!("graphics");
  }

  // Perl revtex4.cls.ltxml L60-62: auto-load `<jobname>.rty` if present.
  // Papers like cond-mat0201306 stash paper-local macros (`\TR`, `\GC`,
  // `\bracketOpen` etc.) in this file via revtex's runtime convention.
  Digest!("\\InputIfFileExists{\\jobname.rty}{}{}")?;
});
