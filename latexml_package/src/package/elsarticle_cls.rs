use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: elsarticle.cls.ltxml
  // Generally ignorable options
  for option in [
    "12pt", "11pt", "10pt", "endfloat", "endfloats", "numafflabel",
    "oneside", "twoside", "onecolumn", "twocolumn",
    "lefttitle", "centertitle", "reversenotenum",
    "symbold", "ussrhead", "nameyear",
    "doublespacing", "reviewcopy"].iter()
  {
    DeclareOption!(*option, None);
  }
  // elsarticle.cls defines `\newif\ifpreprint`, set by its options (true for `preprint` and `review`, false for
  // `final` and the journal layouts) and true by default (`\ExecuteOptions{…,preprint,…}`, elsarticle.cls:112).
  // Journal styles test it (ycviu.sty:60, 298); without it they saw an undefined CS and cascaded (2311.04591).
  DefConditional!("\\ifpreprint");
  Digest!("\\global\\preprinttrue")?;
  // elsarticle.cls:40-45 declares its other conditionals, set by its options (:74, 80, 81, 110). The Elsevier journal
  // styles test them while they load — jasr.sty:327, 356 `\iflongmktitle` inside the `\if@twocolumn` branches of its
  // `\maketitle` (cnf.sty, jcomp.sty alike): undefined, they unbalanced TeX's skip of the false branch, so the other
  // branch's `\maketitle` body ran at load (2609.23725, 23732, 27838, 39431, 09773, 31741, 39502). The one exception is
  // `\ifuseexplthreefunctions`, which the class sets when expl3.sty exists (:49-51): it stays false here, since the
  // class's expl3 affiliation helpers it would select (:327-) are not emulated.
  RawTeX!(r"\newif\ifnonatbib\newif\iflongmktitle\newif\ifnopreprintline\newif\ifdoubleblind\newif\ifuseexplthreefunctions");
  DeclareOption!("nonatbib", { Digest!("\\global\\nonatbibtrue")?; });
  DeclareOption!("nopreprintline", { Digest!("\\global\\nopreprintlinetrue")?; });
  DeclareOption!("longtitle", { Digest!("\\global\\longmktitletrue")?; });
  DeclareOption!("doubleblind", { Digest!("\\doubleblindtrue")?; });
  // elsarticle.cls:211-235: `\emailauthor{<email>}{<name>}` and `\urlauthor`, which `\ead` writes to the .aux and a
  // paper may call itself (2609.09773, 31741, 39502, before or after the author it names). The class collects them into
  // the first page's "Email address:" note as "<email> (<name>)": a frontmatter note, as printed, which no author's
  // contact would place right.
  DefMacro!("\\emailauthor Semiverbatim {}", "\\@add@frontmatter{ltx:note}[role=email]{\\texttt{#1} (#2)}");
  DefMacro!("\\urlauthor Semiverbatim {}", "\\@add@frontmatter{ltx:note}[role=url]{\\texttt{#1} (#2)}");
  // The Elsevier journal styles' `\KWD` (jasr.sty:666 "Keywords:"), defined only inside their first-page printer,
  // which the frontmatter replaces: the keywords keep their label (2609.23725, 23732, 27838).
  def_macro_noop("\\KWD")?;
  // elsarticle.cls:71-87: the journal layout, `\jtype` — `\def\jtype{0}` at load, `\xdef`'d by `preprint` (0),
  // `5p`, `3p`, `1p` — declared in the class's order, so the last declared of several wins. With article's
  // `\if@twocolumn` it decides fleqn below. The macro is the class's: journal styles test it (ecrc.sty:7, 23
  // `\ifnum\jtype=1`; 170 run-329 papers, 1011.4942).
  DefMacro!("\\jtype", "0");
  DeclareOption!("preprint", {
    Digest!("\\xdef\\jtype{0}\\global\\preprinttrue")?;
    assign_value("@elsarticle@jtype", 0i64, Scope::Global);
  });
  // In the class's order (elsarticle.cls:71-76): `final` and `review` after `preprint`.
  DeclareOption!("final", { Digest!("\\global\\preprintfalse")?; });
  DeclareOption!("review", { Digest!("\\global\\preprinttrue")?; });
  DeclareOption!("5p", {
    Digest!("\\xdef\\jtype{5}\\global\\preprintfalse")?;
    assign_value("@elsarticle@jtype", 5i64, Scope::Global);
  });
  DeclareOption!("3p", {
    Digest!("\\xdef\\jtype{3}\\global\\preprintfalse")?;
    assign_value("@elsarticle@jtype", 3i64, Scope::Global);
  });
  DeclareOption!("1p", {
    Digest!("\\xdef\\jtype{1}\\global\\preprintfalse")?;
    assign_value("@elsarticle@jtype", 1i64, Scope::Global);
  });
  // Perl L28: times option pulls in txfonts
  DeclareOption!("times", {
    RequirePackage!("txfonts");
  });
  // Perl L30-32: flags for later conditional behaviour
  DeclareOption!("seceqn", { assign_value("@seceqn", 1i64, Scope::Global); });
  DeclareOption!("secthm", { assign_value("@secthm", 1i64, Scope::Global); });
  DeclareOption!("amsthm", { assign_value("@amsthm", 1i64, Scope::Global); });
  // Perl L33-35: natbib defaults
  DeclareOption!("authoryear", { assign_value("@biboptions", Stored::from("round,authoryear"), Scope::Global); });
  DeclareOption!("number", { assign_value("@biboptions", Stored::from("numbers"), Scope::Global); });
  DeclareOption!("numbers", { assign_value("@biboptions", Stored::from("numbers"), Scope::Global); });
  // Pass other options to article
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  ProcessOptions!();
  LoadClass!("article");
  // Beyond-Perl fidelity (OXIDIZED_DESIGN #50): real elsarticle.cls L47 does an
  // unconditional `\RequirePackage[T1]{fontenc}`, so `<`/`>`/etc. are literal in
  // the PDF. Establish T1 so we don't fall back to OT1 (`<`->¡, `>`->¿). Perl
  // leaves it at OT1; divergence from Perl.
  RequirePackage!("fontenc", options => vec!["T1".to_string()]);
  RequirePackage!("elsart_support_core");
  // DIVERGENCE from Perl (#335, KPE #305): elsarticle.cls inputs fleqn.clo
  // only for `5p`, or `3p` with `twocolumn` (elsarticle.cls:1280, 1295); the
  // default `preprint`, `1p` and one-column `3p` centre their equations.
  // Perl's binding loads fleqn for every layout (2605.05858).
  let jtype = lookup_int("@elsarticle@jtype");
  if jtype == 5 || (jtype == 3 && if_condition(&T_CS!("\\if@twocolumn"))?.unwrap_or(false)) {
    RequirePackage!("fleqn");
  }
  RequirePackage!("graphicx");
  RequirePackage!("pifont");
  // natbib with biboptions, unless the `nonatbib` option (elsarticle.cls:1242-1244): those papers bring biblatex or
  // another citation package of their own (2609.05849, 20719, 26003 biblatex; 19225, 23461 apacite). Perl's binding
  // loads natbib for every paper (KNOWN_PERL_ERRORS #501).
  let nonatbib = if_condition(&T_CS!("\\ifnonatbib"))?.unwrap_or(false);
  if !nonatbib {
    let natbib_opts_stored = lookup_value("@biboptions");
    let natbib_opts = match &natbib_opts_stored {
      Some(Stored::String(s)) => with(*s, |s| s.to_string()),
      _ => "numbers".to_string(),
    };
    let natbib_opt_vec: Vec<String> = natbib_opts.split(',').map(|s| s.trim().to_string()).collect();
    RequirePackage!("natbib", options => natbib_opt_vec);
  }
  RequirePackage!("hyperref");
  // elsarticle.cls:1247-1249 writes `\biboptions` to the .spl for natbib's options on the next run; without natbib
  // they select nothing.
  if nonatbib {
    def_macro_noop("\\biboptions{}")?;
  } else {
    DefMacro!("\\biboptions{}", "\\setcitestyle{#1}");
  }

  // Perl L58-67: override {enumerate}/{itemize} to accept optional arg
  DefEnvironment!("{enumerate}[]",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    mode => "internal_vertical", locked => true,
    properties => { begin_itemize("enumerate", Some("enum"), BeginItemizeOptions::default())? });
  // Perl binds the itemize to the `enum` counter (elsarticle.cls.ltxml:64, "not even sure what the
  // intended effect is"), so every item printed `1.`/`(a)`; elsarticle.cls:1143-1150 labels an
  // itemize by `\labelitem\romannumeral\the\@itemdepth` like the kernel. SURPASS (OXIDIZED_DESIGN
  // #346, KNOWN_PERL_ERRORS #360), pdflatex the oracle. Repro:
  // list-structure/elsarticle_itemize_has_bullets. Its optional argument is the level's label
  // (`\@Itemize[#1]` defines `\labelitem<\@itemdepth>`, elsarticle.cls:1143-1147).
  DefEnvironment!("{itemize} OptionalUndigested",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    mode => "internal_vertical", locked => true,
    properties => { begin_itemize("itemize", Some("@item"), BeginItemizeOptions::default())? },
    after_digest_begin => sub[whatsit] {
      if let Some(arg) = whatsit.get_arg(1) {
        set_itemization_style(arg.raw_tokens(), None)?;
      }
    });

  // Newer elsarticle.cls (2018+) added {graphicalabstract} and {highlights}
  // for Elsevier journal submissions. Real templates wrap a TikZ figure or
  // bullet list inside these and Elsevier's typesetter renders separately
  // from the main body. For LaTeXML's HTML output, treat them as
  // semantically-tagged note blocks. Driver: 1907.06674.
  DefEnvironment!("{graphicalabstract}",
    "<ltx:note role='graphicalabstract'>#body</ltx:note>",
    mode => "internal_vertical", locked => true);
  DefEnvironment!("{highlights}",
    "<ltx:note role='highlights'>#body</ltx:note>",
    mode => "internal_vertical", locked => true);
});
