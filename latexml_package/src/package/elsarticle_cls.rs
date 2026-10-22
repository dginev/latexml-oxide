use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};

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
  // `\ifuseexplthreefunctions`, which the class sets when expl3.sty exists (:49-51): it stays false here, as the code it
  // selects (:327-) is not read; its affiliation keys are what `\affiliation` reads (`stm_affiliation`,
  // elsart_support_core_sty.rs).
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
  // elsarticle.cls:1200-1236 `\newproof`: the environment steps a counter as `\newtheorem` does (`\@prf` is
  // `\refstepcounter`, so a `\label` in it names that number; `\newproof{x}[y]{…}` steps y's, `\@oprf`; `{…}[within]`
  // numbers within it, `\@xnprf`), but its head (`\@beginproof`, `\@opargbeginproof`) prints only the name, in small
  // caps with a period ("Proof.", "Proof (note)."; :1229-1233), and the body is upright. Perl lets `\newproof` be
  // `\newtheorem` (elsart_support_core.sty.ltxml:208), bold heads "Proof 1." with an italic body (1304.5109,
  // 2606.18393's own numbered `\newtheorem{proof}`; KNOWN_PERL_ERRORS #1080). Here a numbered theorem whose title
  // prints the name alone. Each form declares under `\expandafter\@ifdefinable\csname #1\endcsname` (:1205, :1213,
  // :1220): a name already defined (amsthm's `proof`, loaded first) is LaTeX's "Command \proof already defined", and
  // the first definition stays (the reviewer's probe el2: pdflatex 1 error, amsthm's "Proof."). Guards
  // `perfect_kernel_batch78n::{elsarticle_newproof_head_is_unnumbered, elsarticle_newproof_steps_its_counter,
  // elsarticle_newproof_keeps_a_defined_proof}`.
  DefPrimitive!("\\newproof {}[]{}[]", sub[(thmset, otherthmset, typ, within)] {
    let name = thmset.to_string();
    if !engine::latex_constructs::is_definable_latex(&T_CS!(s!("\\{name}")))?.0 {
      let mut refusal = vec![T_CS!("\\expandafter"), T_CS!("\\@ifdefinable"), T_CS!("\\csname")];
      refusal.extend(ExplodeText!(name));
      refusal.extend([T_CS!("\\endcsname"), T_BEGIN!(), T_END!()]);
      Digest!(Tokens::new(refusal))?;
      return Ok(Vec::new());
    }
    engine::latex_constructs::define_new_theorem(
      None,
      thmset,
      otherthmset,
      Some(typ),
      within,
      Some(Tokenize!("\\rmfamily")),
    )?;
    DefMacro!(
      T_CS!(s!("\\format@title@{name}")),
      parse_parameters("{}", &T_CS!(s!("\\format@title@{name}")), true)?,
      Some(ExpansionBody::Tokens(mouth::tokenize_internal(TeXString::assembled(s!(
        "{{\\normalfont\\scshape\\lx@tag{{\\csname lx@name@{name}\\endcsname}}{{\\ifx.#1.\\else\\space(#1)\\fi}}.}}"
      ))))),
      scope => Some(Scope::Global)
    );
  });
  // DIVERGENCE from Perl (#335, KPE #305): elsarticle.cls inputs fleqn.clo
  // only for `5p`, or `3p` with `twocolumn` (elsarticle.cls:1280, 1295); the
  // default `preprint`, `1p` and one-column `3p` centre their equations.
  // Perl's binding loads fleqn for every layout (2605.05858).
  let jtype = lookup_int("@elsarticle@jtype");
  if jtype == 5 || (jtype == 3 && if_condition(&T_CS!("\\if@twocolumn"))?.unwrap_or(false)) {
    RequirePackage!("fleqn");
  }
  // elsarticle.cls:1252-1300: the journal layouts `1p`, `3p` and `5p` load geometry and set the page with it, so
  // a document may call `\geometry`, `\newgeometry` and `\restoregeometry` (witnesses 2609.13883, 2404.14059,
  // 2103.06107, 1904.12395). Perl's binding has the load commented out (elsarticle.cls.ltxml:44,
  // KNOWN_PERL_ERRORS #629). The class's page size and one-column text width (1p: 384pt; 3p: 468pt) reach the
  // geometry binding, which sizes measured SVG pictures by them (OXIDIZED_DESIGN #99); the two-column layouts
  // (5p, and 3p with `twocolumn`) keep the class default, since our geometry has no column model and a full-width
  // picture there is a column wide (OXIDIZED_DESIGN #561).
  if matches!(jtype, 1 | 3 | 5) {
    RequirePackage!("geometry");
    let twocolumn = jtype == 5 || if_condition(&T_CS!("\\if@twocolumn"))?.unwrap_or(false);
    if !twocolumn {
      let (textheight, textwidth, footskip) = if jtype == 1 { ("562pt", "384pt", "12pt") } else { ("622pt", "468pt", "18pt") };
      Digest!(mouth::tokenize_internal(TeXString::assembled(s!(
        "\\geometry{{twoside,paperwidth=210mm,paperheight=297mm,textheight={textheight},\
         textwidth={textwidth},centering,headheight=50pt,headsep=12pt,footskip={footskip}}}"
      ))))?;
    }
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
    // (elsarticle.cls:1243 `\RequirePackage[\@biboptions]{natbib}`: the class's own load, so a document's repeat with
    // other options is LaTeX's "Option clash" — binding/content.rs `apply_new_options_on_reload`)
    AssignValue!("natbib.sty:requested_by_binding" => false, Some(Scope::Global));
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
  // elsarticle.cls (TL 2025 :153-176), the marks and notes under `\fnref`/`\corref`, which papers call directly (run
  // 340: `\fnmark` 4 of 35 sampled papers that ship elsarticle.cls, `\cormark` 3, `\nonumnote` 3): `\fnmark[<keys>]`
  // and `\cormark[<key>]` mark the author as `\fnref`/`\corref` do (:165 `\def\fnref#1{\fnmark[#1]}`), and
  // `\nonumnote{<text>}` (:153-155) adds an unnumbered title-page footnote: a note. Perl's elsarticle binding lacks
  // them. Witnesses 2609.13494, 2302.09410, 2106.01129.
  DefMacro!("\\fnmark[]", "\\fnref{#1}");
  DefMacro!("\\cormark[]", "\\corref{#1}");
  DefMacro!("\\nonumnote{}", "\\@add@frontmatter{ltx:note}[role=thanks]{#1}");
  // The 2020+ copies' `\newpageafter{title|author|abstract}` (16 of 35 sampled), a page break in the title block: a
  // guess a paper's own definition replaces.
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    RawTeX!(r"\def\newpageafter#1{\gdef\@elsarticlenewpageafter{#1}}");
  }
  // elsarticle.cls (TL 2025, v3.4c) internals and setters papers call (run-347 inventory: 122 papers with one
  // undefined), as the class defines them:
  // - :145 `\tnotemark[<keys>]`, the title's note marks, as `\tnoteref` (:146; 10 papers; 2101.08044, 2203.02441).
  // - :1021-1027 `\myfooter[<align>]<text>␣` (its text delimited by a space, as the class's) and `\myfooterfont`, the
  //   first page's footer: page furniture, stored (15 papers; 2209.12222, 2208.12518).
  // - :671-673 `\abstracttitle{<title>}`, the abstract's heading (12 papers; 2204.07076, 2104.02951): the class's
  //   `{abstract}` prints `\@elsarticleabstitle` (:676; older copies a literal "Abstract"), never `\abstractname`, so the
  //   frontmatter abstract takes its name from there (`\lx@abstract@name`), and `\abstractname` stays the document's;
  //   :737-740 `\keywordtitle`/`\keywordtitlesep`, stored.
  // - `\@journal` stays empty until `\journal` sets it (elsart_support_core_sty.rs): the class's :32 "Elsevier Ltd" is
  //   only its `\ProvidesClass` line's, :1302 `\let\@journal\@empty` resets it.
  // - :192-193 `\resetTitleCounters` over the title-note counters `tnote`, `fnote`, `cnote` (:150, :157, :182; 8; 2407.02308).
  // - :670, :736 `\absbox`/`\keybox` and :755-756 `\Columnwidth`, which papers' own title code fills and reads (6; 1311.6418).
  // - :126-131 `\elsLabel`/`\Newlabel`/`\elsRef`, the class's note labels (5; 2011.11672, 2602.18725), :117 `\comma`, :249 `\@eadauthor`,
  //   :242 `\pprinttitle`, :197-198 `\hashchar`/`\underscorechar`, :1073-1076 `\elsparagraph`.
  // Perl's binding defines none of them (KNOWN_PERL_ERRORS #723).
  DefMacro!("\\tnotemark[]", "\\tnoteref{#1}");
  RawTeX!(
    r"\let\@elsarticlemyfooter\@empty
\let\@elsarticlemyfooteralign\@empty
\def\myfooter[#1]#2 {\gdef\@elsarticlemyfooteralign{#1}\gdef\@elsarticlemyfooter{#2}}
\def\myfooterfont#1{\gdef\@myfooterfont{#1}}
\myfooterfont{\footnotesize\itshape}
\def\abstracttitle#1{\gdef\@elsarticleabstitle{#1}}
\abstracttitle{Abstract}
\def\lx@abstract@name{\format@title@abstract{\@elsarticleabstitle}}
\def\keywordtitle#1{\gdef\@elsarticlekwdtitle{#1}}
\def\keywordtitlesep#1{\gdef\@elsarticlekeywordtitlesep{#1}}
\keywordtitle{Keywords}
\keywordtitlesep{:\ }
\newcounter{tnote}\newcounter{fnote}\newcounter{cnote}
\def\resetTitleCounters{\c@cnote=0
   \c@fnote=0 \c@tnote=0 \c@footnote=0}
\newbox\absbox
\newbox\keybox
\newdimen\Columnwidth
\Columnwidth=\columnwidth
\def\elsLabel#1{\@bsphack\protected@write\@auxout{}%
         {\string\Newlabel{#1}{\@currentlabel}}\@esphack}
\def\Newlabel#1#2{\expandafter\xdef\csname X@#1\endcsname{#2}}
\def\elsRef#1{\@ifundefined{X@#1}{0}{\csname X@#1\endcsname}}
\let\comma\@empty
\newtoks\@eadauthor
\def\pprinttitle{}
\def\hashchar{\expandafter\@gobble\string\~}
\def\underscorechar{\expandafter\@gobble\string\_}
\newcommand\elsparagraph{\@startsection{paragraph}{4}{0\z@}%
           {10\p@ \@plus 6\p@ \@minus 3\p@}%
           {-6\p@}%
           {\normalfont\itshape}}"
  );
  // The older releases papers ship (run-347 inventory: 225 papers whose own elsarticle.cls defines a command this
  // binding lacked; 100 of the 225 copies are v1.21, 2010/06/11, 54 v1.20b), as v1.21 defines them (1303.2653's and
  // 1404.3867's copies; v3.x has none of them, so they are a release's guesses the document's own definition replaces,
  // `DefinitionOrigin::Fallback`):
  // - :238-239 `\email{<address>}`, an e-mail line under the authors, where v3 has `\ead` (85 papers; 1305.6441,
  //   1306.0018, 1404.3867): the last author's e-mail contact, as `\ead`.
  // - :234-236 `\correspondingauthor[<mark>]{<text>}`, a footnote naming the corresponding author: a note (8).
  // - :844 `{nomenclature}`, a boxed "Nomenclature" section (12; 1411.2181, 2103.08145), set as that section; :845-863
  //   `{deflist}` with `\defitem`/`\defterm`, a hanging definition list (13; 1303.2653, 1811.08269).
  // - 1404.3867's :797-808 `\normalMode`, which takes back `\appendix`'s numbering (23 papers; 2404.00783, 1411.0092): the
  //   appendices end there (`end_appendices`, as appendix.sty's `{appendices}` ends), and the sections after it are
  //   sections again.
  // The astronomical journal abbreviations (`\apj`, `\mnras`, `\aap`… 7 to 16 papers each) are aas_macros.sty pasted
  // into individual copies (0812.4232 `\def\apj{ApJ}`, 1504.02326, 1807.02064 `\newcommand*\apj{ApJ}`, 2003.05953,
  // 2401.03908 `\newcommand\mnras{\ref@jnl{MNRAS}}`), no elsarticle release's: aas_macros' binding provides them
  // where the paper's copy defines them, and only the names still undefined
  // (`shipped_class::provide_shipped_aas_macros`, shared with PoS, svmult, svjour3 and mdpi).
  // elsarticle.cls:1342-1353 (TL 2025 v3.4c; every v1.2 to v3.x copy papers ship alike): `\appendix` numbers the
  // sections "Appendix A" (`\thesection{\appendixname~\@Alph\c@section}`, the `\ref` text; its subsections "Appendix
  // A.1"), the equations within each appendix "A.1" (`\@addtoreset{equation}{section}`), the figures and tables "A.2"
  // (the letter before the running figure number, not reset). The kernel's `\appendix` (article_cls.rs `\@appendix`)
  // runs first for the `ltx:appendix` units, their tag the section number alone ("Appendix A", not the kernel's
  // `\appendixname` before it; as iopart's, iopart_support_sty.rs). A v1.0.1 copy (2008/05/22; 22 of 297 copies,
  // 0711.2399) has no `\appendix`, so the article's stands where the paper ships one. Perl's elsarticle.cls.ltxml leaves
  // the kernel's ("A", equation 4, figure 2; 2609.03753, 2608.23563): KNOWN_PERL_ERRORS #1098. Guard
  // `perfect_kernel_batch78q::appendix_counters_elsarticle`.
  if crate::shipped_class::shipped_class_text("elsarticle")?
    .as_deref()
    .is_none_or(|text| crate::shipped_class::class_code_contains(text, "\\renewcommand\\appendix"))
  {
    RawTeX!(r"\renewcommand\appendix{\@appendix\par
  \setcounter{section}{0}\setcounter{subsection}{0}\setcounter{equation}{0}%
  \gdef\thefigure{\@Alph\c@section.\arabic{figure}}\gdef\thetable{\@Alph\c@section.\arabic{table}}%
  \gdef\thesection{\appendixname~\@Alph\c@section}\@addtoreset{equation}{section}%
  \gdef\theequation{\@Alph\c@section.\arabic{equation}}%
  \gdef\fnum@appendix{\thesection}\gdef\typerefnum@appendix{\thesection}}");
  }
  crate::shipped_class::provide_shipped_aas_macros("elsarticle")?;
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    DefMacro!("\\email{}", "\\lx@add@contact[role=email]{#1}");
    DefMacro!(
      "\\correspondingauthor[]{}",
      "\\@add@frontmatter{ltx:note}[role=corresponding]{#2}"
    );
    DefConstructor!("\\lx@elsarticle@endappendices", sub[document, _args, _props] {
      document.maybe_close_element("ltx:appendix")?;
    },
      before_digest => {
        end_appendices();
      });
    RawTeX!(
      r"\newenvironment{nomenclature}{\par\section*{Nomenclature}}{\par}
\def\deflist{\@ifnextchar[{\@deflist}{\@deflist[]}}
\newbox\defbox%
\newdimen\defboxdimen%
\def\@deflist[#1]{%
\setbox\defbox\hbox{#1:\quad}%
\defboxdimen\wd\defbox%
\def\tempa{#1}%
\par\addvspace{12pt plus2pt minus2pt}%
\setlength\parindent{0pt}%
\def\deftitle##1{{\noindent\itshape##1}\par}%
\ifx\tempa\@empty%
\def\defitem##1{\@hangfrom{##1\ }}%
\else%
\def\defitem##1{\@hangfrom{\hbox to \defboxdimen{##1\quad}}}%
\fi%
\def\defterm##1{##1\par}
}
\def\enddeflist{\par\addvspace{8pt plus2pt minus2pt}}
\newcommand\normalMode{\par\lx@elsarticle@endappendices
  \setcounter{section}{0}%
  \setcounter{subsection}{0}%
  \setcounter{equation}{0}
  \gdef\thefigure{\@Alph\c@section.\arabic{figure}}%
  \gdef\thetable{\@Alph\c@section.\arabic{table}}%
  \gdef\thesection{\@arabic\c@section}%
  \gdef\thesubsection{\c@section.\arabic{subsection}}%
  \@addtoreset{equation}{section}%
  \gdef\theequation{\@Alph\c@section.\arabic{equation}}%
  \addtocontents{toc}{\string\let\string\numberline\string\tmptocnumberline}{}{}
}"
    );
  }
});
