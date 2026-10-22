use crate::prelude::*;
#[rustfmt::skip]
LoadDefinitions!({
  // Perl: revtex4-1.cls.ltxml
  for option in ["overload", "showpacs", "noshowpacs",
    "showkeys", "noshowkeys", "balancelastpage", "nobalancelastpage",
    "bibnotes", "nobibnotes",
    "footinbib", "nofootinbib", "altaffilletter", "altaffilsymbol",
    "superbib", "citeautoscript", "raggedbottom", "flushbottom",
    "tightenlines", "lengthcheck",
    "fleqn", "floats", "endfloats",
    // Perl L29 also declares the starred `endfloats*`; Rust was missing it
    // so `\documentclass[endfloats*]{revtex4-1}` fell through to the article
    // option passthrough.
    "endfloats*",
    "titlepage", "notitlepage",
    "groupedaddress", "unsortedaddress", "runinaddress",
    "superscriptaddress", "byrevtex", "floatfix", "nofloatfix",
    "ltxgridinfo", "outputdebug", "raggedfooter",
    "newabstract", "oldabstract"].iter()
  {
    DeclareOption!(*option, None);
  }
  // The options that decide whether the class prints the `\preprint` numbers (revtex4-1.cls:5856-5909, revtex4.cls
  // :3847-3892, revtex4-2.cls alike): `preprint` (and `manuscript`, `checkin`) set the preprint style, which prints
  // them unless `nopreprintnumbers`; `preprintnumbers` prints them in any style (revtex4_support_sty.rs
  // `preprint_printed`).
  for option in ["preprint", "manuscript", "checkin"] {
    DeclareOption!(option, {
      assign_value("revtex@preprintsty", true, Some(Scope::Global));
    });
  }
  DeclareOption!("preprintnumbers", {
    assign_value("revtex@preprintnumbers", Stored::Bool(true), Some(Scope::Global));
  });
  DeclareOption!("nopreprintnumbers", {
    assign_value("revtex@preprintnumbers", Stored::Bool(false), Some(Scope::Global));
  });
  // The society and journal, which the section numbers depend on (the `rmp` journal, apsrmp4-1.rtx:115-139; PRL's
  // `secnumdepth`, aps4-1.rtx:564), are read from the options as the class reads them
  // (`revtex4_support_sty::selected_journal`).
  for substyle in ["aps", "pra", "prb", "prc", "prd", "pre", "prl",
    "prstab", "rmp", "osa", "osameet", "opex", "tops", "josa"].iter()
  {
    DeclareOption!(*substyle, None);
  }
  // The options that number the sections and equations (`revtex4_support_sty::define_section_numbers`): `eqsecnum`
  // (revtex4-1.cls:6036-6043) and `secnumarabic` (:6047-6050).
  for option in ["eqsecnum", "secnumarabic"] {
    DeclareOption!(option, {
      assign_value(&s!("revtex@opt@{option}"), true, Some(Scope::Global));
    });
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
      assign_value(&s!("revtex_load_{}", pkg_owned), true, Some(Scope::Global));
    });
    let nopkg = s!("no{}", pkg);
    DeclareOption!(&nopkg, {
      assign_value(&s!("revtex_load_{}", pkg), false, Some(Scope::Global));
    });
  }
  // Perl L47-49: osajnl (Optical Society) sub-option also pushed graphics
  // onto @revtex_toload and Let-aliased \ocis → \pacs. Revtex4 class has
  // this (revtex4_cls.rs L37-39); the revtex4-1 port had dropped it,
  // which breaks OSA documents that rely on the \ocis classification.
  DeclareOption!("osajnl", {
    assign_value("revtex_load_graphics", true, Some(Scope::Global));
    DefMacro!("\\ocis", "\\pacs");
  });
  // The `aip` society (revtex4-1.cls:6317 `\@process@society`) picks AIP's
  // bibliography style (`implicit_bibstyle`); the class ignores it otherwise.
  DeclareOption!("aip", {
    assign_value("revtex_society", pin("aip"), Some(Scope::Global));
  });
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });
  // REVTeX4-1 declares `author-year` / `numerical`, DEFAULT numerical
  // (revtex4-1.cls L6001-6003 → `\@booleanfalse\authoryear@sw`): bracketed
  // numeric citations like the PDF. Perl's revtex4-1 binding and natbib's
  // default both give author-year, so oxide SURPASSES Perl to match the real
  // class + PDF; a later `\setcitestyle`/`\bibpunct` still overrides. Sister
  // logic to revtex4_cls.rs. html_feedback #6609.
  DeclareOption!("numerical", {
    assign_value("revtex_cite_style", pin("numbers"), Some(Scope::Global));
  });
  DeclareOption!("author-year", {
    assign_value("revtex_cite_style", pin("authoryear"), Some(Scope::Global));
  });
  // revtex4-1.cls:6186-6193 `\DeclareOption{linenumbers}{\appdef\class@documenthook{\RequirePackage{lineno}…\linenumbers}}`:
  // the option loads lineno at the document's start, and papers given it call `\nolinenumbers`, `{linenomath}`
  // (revtex4-1 and revtex4-2 papers of run 353: 16 + 16 with the option and lineno's commands undefined; 1010.3732,
  // 1209.2541, 2003.07846, 2010.12920). Without the option the class loads none: a paper calling them then errs in
  // pdflatex too. Perl's binding lacks the option (KNOWN_PERL_ERRORS #697).
  DeclareOption!("linenumbers", {
    assign_value("revtex_linenumbers", true, Some(Scope::Global));
  });
  // revtex4-1.cls:6194-6198, revtex4-2.cls:6215-6219: `nomerge` turns natbib's entry merging off.
  DeclareOption!("nomerge", {
    assign_value("revtex_nomerge", true, Some(Scope::Global));
  });
  assign_value("revtex_cite_style", pin("numbers"), Some(Scope::Global));
  ProcessOptions!();
  LoadClass!("article");
  // Queue the citation style onto natbib (numbers → numeric `[N]`; author-year →
  // natbib's default). `\PassOptionsToPackage` on purpose, not a direct
  // `RequirePackage("natbib", [numbers])`: pre-loading natbib here flips it ahead
  // of revtex4_support's hyperref and that perturbation trips a latent libxml
  // heisenbug. Queuing keeps natbib in its original position. Sister logic to
  // revtex4_cls.rs. html_feedback #6609.
  if lookup_string("revtex_cite_style") == "numbers" {
    Digest!("\\PassOptionsToPackage{numbers}{natbib}")?;
  }
  // revtex4-1.cls:7021 `\RequirePackage[sort&compress]{natbib}`: a citation's numbers sorted, a run of three or more
  // joined with an en dash ("[1–3]"; natbib_sty.rs, KNOWN_PERL_ERRORS #1072). Guard
  // `perfect_kernel_batch78j::revtex_citations_sort_and_compress`.
  Digest!("\\PassOptionsToPackage{sort&compress}{natbib}")?;
  RequirePackage!("revtex4_support");
  // revtex4-1.cls:7046, revtex4-2.cls:7067 `\let\NAT@merge\thr@@`: `\cite{a,*b}` merges b's entry into a's
  // (`natbib_sty::natbib_merge_citations`; REVTeX 4.0 does not merge). 1808.00247.
  if !lookup_bool("revtex_nomerge") {
    natbib_sty::natbib_merge_citations("\\thr@@", false)?;
  }
  revtex4_support_sty::define_section_numbers("4-1")?;
  revtex4_support_sty::record_implicit_bibstyle("4-1");
  revtex4_support_sty::retire_option_handlers();
  revtex4_support_sty::define_float_repair()?;

  // Load the AMS packages requested via options BEFORE the `.rty` input.
  // Perl revtex4-1.cls.ltxml runs `map { RequirePackage($_) } @revtex_toload`
  // (L58) *before* the `\jobname.rty` load (L60-63). A paper-local `.rty`
  // that uses an AMS macro — e.g. `\DeclareMathOperator` in HSWS.rty
  // (witness 1508.02642, `\documentclass[…,amsmath,…]{revtex4-1}`) — would
  // otherwise hit it undefined. Rust previously loaded the `.rty` first.
  for pkg in ["amsfonts", "amssymb", "amsmath"].iter() {
    if lookup_bool(&s!("revtex_load_{}", pkg)) {
      RequirePackage!(*pkg);
    }
  }
  if lookup_bool("revtex_load_graphics") {
    RequirePackage!("graphics");
  }

  // Perl revtex4-1.cls.ltxml L60-63: auto-load `<jobname>.rty` if present.
  // Same convention as revtex4 — paper-local macros stashed in .rty file.
  Digest!("\\InputIfFileExists{\\jobname.rty}{}{}")?;
  if lookup_bool("revtex_linenumbers") {
    DefPrimitive!("\\lx@revtex@lineno", {
      RequirePackage!("lineno");
    });
    RawTeX!(r"\AtBeginDocument{\lx@revtex@lineno}");
  }

  // The class internals documents call, `\doi` among them (`define_revtex41_internals`).
  revtex4_support_sty::define_revtex41_internals()?;
});
