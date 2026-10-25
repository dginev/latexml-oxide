use crate::{engine::latex_constructs::is_document_class, prelude::*};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: revtex4.cls.ltxml — RevTeX 4 document class
  // REVTeX 4.0 itself, not the 4.2 this binding also serves (the name it loads under, read before the classes it loads)
  let revtex4_0 = Expand!(Tokens!(T_CS!("\\@currname"))).to_string() == "revtex4";

  // Generally ignorable options
  for option in [
    "overload", "showpacs", "noshowpacs",
    "showkeys", "noshowkeys", "balancelastpage", "nobalancelastpage",
    "bibnotes", "nobibnotes",
    "footinbib", "nofootinbib",
    "superbib", "citeautoscript", "raggedbottom", "flushbottom", "tightenlines",
    "lengthcheck", "fleqn", "floats", "endfloats",
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
  // The front matter's footnote marks (revtex4_support_sty.rs `\frontmatter@thefootnote`): letters under
  // `altaffilletter`, symbols by default and under `altaffilsymbol` (revtex4-1.cls:5955-5957, revtex4.cls:3934-3937,
  // revtex4-2.cls:5976-5978).
  DeclareOption!("altaffilletter", {
    assign_value("revtex@altaffilletter", true, Some(Scope::Global));
  });
  DeclareOption!("altaffilsymbol", {
    assign_value("revtex@altaffilletter", false, Some(Scope::Global));
  });
  // REVTeX 4.2's `aapm` society (aapm4-2.rtx; REVTeX 4.0 has no aapm.rtx). The society and journal, which the section
  // numbers depend on (the `rmp` journal, apsrmp4-2.rtx:124-148, revtex4's rmp.rtx:113-122; `aapm`, aapm4-2.rtx:290-314;
  // PRL's and AIP `cp`'s `secnumdepth`), are read from the options as the class reads them
  // (`revtex4_support_sty::selected_journal`).
  if !revtex4_0 {
    DeclareOption!("aapm", None);
  }
  // The options that number the sections and equations (`revtex4_support_sty::define_section_numbers`): `eqsecnum`
  // (revtex4-2.cls:6057-6064, revtex4.cls:3836-3840) and `secnumarabic` (revtex4-2.cls:6068-6071, revtex4.cls:3987).
  for option in ["eqsecnum", "secnumarabic"] {
    DeclareOption!(option, {
      assign_value(&s!("revtex@opt@{option}"), true, Some(Scope::Global));
    });
  }
  // The options that decide whether the class prints the `\preprint` numbers (revtex4-1.cls:5856-5909, revtex4.cls
  // :3847-3892, revtex4-2.cls alike): `preprint` (and `manuscript`, `checkin`) set the preprint style, which prints
  // them unless `nopreprintnumbers` (in REVTeX 4.0 even then, :3811-3812); `preprintnumbers` prints them in any style
  // (revtex4_support_sty.rs `preprint_printed`).
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

  // The `aip` society (revtex4-2.cls `\@process@society`) picks AIP's
  // bibliography style (`implicit_bibstyle`); the class ignores it otherwise.
  DeclareOption!("aip", {
    assign_value("revtex_society", pin("aip"), Some(Scope::Global));
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
  // revtex4-2.cls:6207-6214 `\DeclareOption{linenumbers}{\appdef\class@documenthook{\RequirePackage{lineno}…\linenumbers}}`:
  // the option loads lineno at the document's start, and papers given it call `\nolinenumbers`, `{linenomath}`
  // (revtex4-1 and revtex4-2 papers of run 353: 16 + 16 with the option and lineno's commands undefined; 1010.3732,
  // 1209.2541, 2003.07846, 2010.12920). Without the option the class loads none: a paper calling them then errs in
  // pdflatex too. Perl's binding lacks the option (KNOWN_PERL_ERRORS #697).
  if !revtex4_0 {
    DeclareOption!("linenumbers", {
      assign_value("revtex_linenumbers", true, Some(Scope::Global));
    });
  }
  if !revtex4_0 {
    // revtex4-1.cls:6194-6198, revtex4-2.cls:6215-6219: `nomerge` turns natbib's entry merging off.
    DeclareOption!("nomerge", {
      assign_value("revtex_nomerge", true, Some(Scope::Global));
    });
  }
  assign_value("revtex_cite_style", pin("numbers"), Some(Scope::Global));
  ProcessOptions!();
  LoadClass!("article");
  // The page a float's panels are set in rows against (the panel grid rule, `declare_panel_page`): REVTeX's, as
  // pdflatex gives it — 510pt in two 246pt columns under `twocolumn` (and revtex4-2's `reprint`; REVTeX 4.0 has no
  // such option), one column of 510pt otherwise; `preprint` (`manuscript`) sets 468pt, in two 229pt columns where
  // `twocolumn` or a `reprint` given before it sets two (`[preprint,twocolumn]`; review rev78x2 finding 4), a later
  // `reprint` undoing it — the options as given, the last standing; `\onecolumngrid`/`\twocolumngrid` regions follow
  // (revtex4_support_sty.rs). Witnesses 1702.02527, 2504.07193 (one column), 2308.02289 (absolute widths against the
  // 246pt column), 2504.20901, 2504.06175 (`\onecolumngrid` regions).
  {
    let reprint: &[&str] = if is_document_class("revtex4") { &[] } else { &["reprint"] };
    let preprint = matches!(
      last_class_option(&[&["preprint", "manuscript"], reprint].concat()),
      Some("preprint" | "manuscript")
    );
    let twocolumn = matches!(
      last_class_option(&[&["twocolumn", "onecolumn"], reprint].concat()),
      Some("twocolumn" | "reprint")
    );
    let (column, text) = if preprint { (229.0, 468.0) } else { (246.0, 510.0) };
    declare_panel_page(column, text, twocolumn);
  }
  // REVTeX's floats are centred (revtex4-2.cls:6770-6772, revtex4.cls:5355-5357
  // `\appdef\@xfloat@prep{\appdef\@parboxrestore{\centering}}`): a `\\` in them is `\@centercr`
  // (`declare_panel_floats_centred`).
  declare_panel_floats_centred();
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
  // revtex4.cls:5480 and revtex4-2.cls:7042 `\RequirePackage[sort&compress]{natbib}`: a citation's numbers sorted, a
  // run of three or more joined with an en dash ("[1–3]"; natbib_sty.rs, KNOWN_PERL_ERRORS #1072). Guard
  // `perfect_kernel_batch78j::revtex_citations_sort_and_compress`.
  Digest!("\\PassOptionsToPackage{sort&compress}{natbib}")?;
  // REVTeX 4.0's own rule for printing a `\preprint` (revtex4_support_sty.rs `preprint_printed`).
  if revtex4_0 {
    assign_value("revtex@preprint@v4.0", true, Some(Scope::Global));
  }
  RequirePackage!("revtex4_support");
  // revtex4-1.cls:7046, revtex4-2.cls:7067 `\let\NAT@merge\thr@@`: `\cite{a,*b}` merges b's entry into a's
  // (`natbib_sty::natbib_merge_citations`; REVTeX 4.0 does not merge). 1808.00247.
  if !revtex4_0 && !lookup_bool("revtex_nomerge") {
    natbib_sty::natbib_merge_citations("\\thr@@", false)?;
  }
  revtex4_support_sty::define_section_numbers(if revtex4_0 { "" } else { "4-2" })?;
  revtex4_support_sty::record_implicit_bibstyle(if revtex4_0 { "" } else { "4-2" });
  revtex4_support_sty::retire_option_handlers();
  if revtex4_0 {
    // revtex4.cls:4092-4094: REVTeX 4.0 groups the addresses unless an address option said otherwise.
    RawTeX!(r"\@ifxundefined\groupauthors@sw{\clo@groupedaddress}{}");
    // `\bm` waits for `\begin{document}` (revtex4.cls:6030-6032; 4.2 defines it at load, revtex4-2.cls:7586).
    revtex4_support_sty::defer_bm_to_begin_document()?;
  } else {
    revtex4_support_sty::define_float_repair()?;
    // The REVTeX 4.2 internals documents call, `\doi` among them (`define_revtex41_internals`).
    revtex4_support_sty::define_revtex41_internals()?;
  }
  // REVTeX 4.0 sets an `\email` or `\homepage` address with `\url`, the `\texttt` its `\href` falls back to
  // (revtex4.cls:4476-4493), so its `_` and `~` read as typed (1001.1076 "aiwen_fan@pku.edu.cn"). REVTeX 4.2, which
  // this binding also serves, sets it in the current font (revtex4-2.cls:1980, its `\href` = `\@href`), as
  // revtex4_support_sty.rs's commands do; `%` other in the address, as `\sanitize@url` makes it (`UrlSemiverbatim`).
  if revtex4_0 {
    DefMacro!("\\email[] UrlSemiverbatim", "\\lx@add@email[name={#1}]{\\lx@address@urlfont{#2}}");
    DefMacro!("\\homepage[] UrlSemiverbatim", "\\lx@add@url[name={#1}]{\\lx@address@urlfont{#2}}");
    DefMacro!("\\@email{} Semiverbatim", "\\endgroup\\lx@add@email[name={#1}]{\\lx@address@urlfont{#2}}");
    DefMacro!("\\@homepage{} Semiverbatim", "\\endgroup\\lx@add@url[name={#1}]{\\lx@address@urlfont{#2}}");
  }

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
  if lookup_bool("revtex_linenumbers") {
    DefPrimitive!("\\lx@revtex@lineno", {
      RequirePackage!("lineno");
    });
    RawTeX!(r"\AtBeginDocument{\lx@revtex@lineno}");
  }

  // Perl revtex4.cls.ltxml L60-62: auto-load `<jobname>.rty` if present.
  // Papers like cond-mat0201306 stash paper-local macros (`\TR`, `\GC`,
  // `\bracketOpen` etc.) in this file via revtex's runtime convention.
  Digest!("\\InputIfFileExists{\\jobname.rty}{}{}")?;
});
