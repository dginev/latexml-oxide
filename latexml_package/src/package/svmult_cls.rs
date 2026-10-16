use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: svmult.cls.ltxml

  // Generally ignorable options
  for option in [
    "nospthms", "vecphys", "vecarrow", "norunningheads", "referee", "oribibl",
    "chaprefs", "footinfo", "openany", "sechang", "deutsch", "francais",
    // These could affect numbering...
    "numart", "book", "envcountresetchap", "envcountresetsect", "envcountsame",
    "envcountchap",
    "natbib",
  ].iter() {
    DeclareOption!(*option, None);
  }
  // svmult.cls:182-208 (svmono.cls:183-210): `graybox` loads color and framed and sets `{svgraybox}` and
  // `{svtintedbox}` shaded; without it they are a quotation-like list (svmult.cls:822-850).
  DeclareOption!("graybox", { AssignValue!("svmult_graybox" => true); });

  // Other options could load sv<option>.clo
  // Anything else gets passed to book.
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{book}")?;
  });

  ProcessOptions!();
  // Perl svmult.cls.ltxml L34: LoadClass('book', withoptions => 1)
  load_class_with_options("book", Tokens!())?;
  RequirePackage!("sv_support");
  RequirePackage!("url");
  // svmult.cls v1.0:939/942 (CR line ends), svmultln.cls:1810/1813: `\newtoks\tocauthor`, `\newtoks\toctitle`, the
  // table-of-contents forms of the author and title — token registers, as in llncs (llncs_cls.rs), so
  // `\toctitle{…}` assigns and a bare `\tocauthor\toctitle\maketitle` never eats what follows. Witnesses 0704.1469,
  // astro-ph0103277, astro-ph9905256, gr-qc0307119, 1504.06158 (svmultln).
  DefRegister!("\\tocauthor" => Tokens!());
  DefRegister!("\\toctitle" => Tokens!());

  // \title with optional * to affect numbering (* => numart, none => book)
  DefMacro!("\\title OptionalMatch:* {}",
    "\\if.#1.\\else\\def\\thesection{\\arabic{section}}\\fi\
     \\lx@add@title{#2}");

  //======================================================================
  // Upright greek letters
  DefMath!("\\ualpha", "\u{03B1}", font => { shape => "upright", forceshape => true });
  DefMath!("\\ubeta",  "\u{03B2}", font => { shape => "upright", forceshape => true });
  DefMath!("\\uchi",   "\u{03C7}", font => { shape => "upright", forceshape => true });
  DefMath!("\\udelta", "\u{03B4}", font => { shape => "upright", forceshape => true });
  DefMath!("\\ugamma", "\u{03B3}", font => { shape => "upright", forceshape => true });
  DefMath!("\\umu",    "\u{03BC}", font => { shape => "upright", forceshape => true });
  DefMath!("\\unu",    "\u{03BD}", font => { shape => "upright", forceshape => true });
  DefMath!("\\upi",    "\u{03C0}", font => { shape => "upright", forceshape => true });
  DefMath!("\\utau",   "\u{03C4}", font => { shape => "upright", forceshape => true });

  // Italic var-Greek letters
  DefMath!("\\varDelta",   "\u{0394}", font => { shape => "italic" });
  DefMath!("\\varGamma",   "\u{0393}", font => { shape => "italic" });
  DefMath!("\\varLambda",  "\u{039B}", font => { shape => "italic" });
  DefMath!("\\varOmega",   "\u{03A9}", font => { shape => "italic" });
  DefMath!("\\varPhi",     "\u{03A6}", font => { shape => "italic" });
  DefMath!("\\varPi",      "\u{03A0}", font => { shape => "italic" });
  DefMath!("\\varPsi",     "\u{03A8}", font => { shape => "italic" });
  DefMath!("\\varSigma",   "\u{03A3}", font => { shape => "italic" });
  DefMath!("\\varTheta",   "\u{0398}", font => { shape => "italic" });
  DefMath!("\\varUpsilon", "\u{03A5}", font => { shape => "italic" });
  DefMath!("\\varXi",      "\u{039E}", font => { shape => "italic" });

  // Blackboard bold letters (identical \bbbX cluster to llncs_cls.rs).
  // Perl: DefPrimitiveI('\bbbc', undef, "\x{2102}");
  // DP audit flags 13 DefPrimitiveI↔DefConstructor mismatches here as
  // structural pattern — see llncs_cls.rs for the shared rationale
  // (Rust's DefConstructor is the idiomatic shape for literal-glyph
  // output with explicit horizontal-mode entry).
  DefConstructor!("\\bbbc",   "\u{2102}",   enter_horizontal => true);
  DefConstructor!("\\bbbf",   "\u{1D53D}",  enter_horizontal => true);
  DefConstructor!("\\bbbh",   "\u{210D}",   enter_horizontal => true);
  DefConstructor!("\\bbbk",   "\u{1D542}",  enter_horizontal => true);
  DefConstructor!("\\bbbm",   "\u{1D544}",  enter_horizontal => true);
  DefConstructor!("\\bbbn",   "\u{2115}",   enter_horizontal => true);
  DefConstructor!("\\bbbone", "\u{1D7D9}",  enter_horizontal => true);
  DefConstructor!("\\bbbp",   "\u{2119}",   enter_horizontal => true);
  DefConstructor!("\\bbbq",   "\u{211A}",   enter_horizontal => true);
  DefConstructor!("\\bbbr",   "\u{211D}",   enter_horizontal => true);
  DefConstructor!("\\bbbs",   "\u{1D54A}",  enter_horizontal => true);
  DefConstructor!("\\bbbt",   "\u{1D54B}",  enter_horizontal => true);
  DefConstructor!("\\bbbz",   "\u{2124}",   enter_horizontal => true);

  // Math operators
  DefMath!("\\getsto", "\u{21C6}", role => "ARROW");
  DefMath!("\\lid",    "\u{2266}", role => "RELOP", meaning => "less-than-or-equals");
  DefMath!("\\gid",    "\u{2267}", role => "RELOP", meaning => "greater-than-or-equals");
  DefMath!("\\grole",  "\u{2277}", role => "RELOP", meaning => "greater-than-or-less-than");
  Let!("\\qedsymbol", "\\qed");

  // Special signs and characters
  DefMacro!("\\D", "\\mathrm{d}");
  DefMacro!("\\E", "\\mathrm{e}");
  Let!("\\eul", "\\E");
  DefMacro!("\\I", "{\\rm i}");
  Let!("\\imag", "\\I");

  // Size and style macros
  DefMacro!("\\partsize",    "\\Large");
  DefMacro!("\\partstyle",   "\\bfseries\\boldmath");
  DefMacro!("\\chapsize",    "\\Large");
  DefMacro!("\\chapstyle",   "\\bfseries\\boldmath");
  DefMacro!("\\secsize",     "\\large");
  DefMacro!("\\secstyle",    "\\bfseries\\boldmath");
  DefMacro!("\\subsecsize",  "\\normalsize");
  DefMacro!("\\subsecstyle", "\\bfseries\\boldmath");
  // svmult.cls:701-710: run-in headings, level-4 `\@startsection`s set bold (italic for the sub form); 1805.00023.
  RawTeX!(r"\newcommand\runinhead{\@startsection{paragraph}{4}{\z@}{-6\p@}{-6\p@}{\normalfont\normalsize\bfseries\boldmath\rightskip=\z@ \@plus 8em\pretolerance=10000 }}
\newcommand\subruninhead{\@startsection{paragraph}{4}{\z@}{-6\p@}{-6\p@}{\normalfont\normalsize\itshape\rightskip=\z@ \@plus 8em\pretolerance=10000 }}");

  def_macro_noop("\\chaptermark{}")?;
  def_macro_noop("\\sectionmark{}")?;
  def_macro_noop("\\subsectionmark{}")?;
  DefMacro!("\\tocauthorstyle",   "\\itshape");
  DefMacro!("\\toctitlestyle",    "\\bfseries");
  DefMacro!("\\tocaftauthskip",   "\\z@");

  // svmult.cls:668 (v5.4 2007 to v5.6 2018; svmono.cls:554 the same): the title is optional, `\prefacename` by
  // default. Perl's `\preface{}` (svmult.cls.ltxml:111) took the next token for it: the bare `\preface` of a blank
  // line titled the chapter `\par` (1001.3294, 1508.02595, svmono).
  DefMacro!("\\prefacename", "Preface");
  RawTeX!(r"\newcommand{\preface}[1][\prefacename]{\chapter*{#1}}");
  // svmult.cls:669-674 (svmono.cls:555-559): the other unnumbered chapters of the front and back matter (1003.4850,
  // 0912.2110).
  DefMacro!("\\forewordname", "Foreword");
  RawTeX!(r"\newcommand{\foreword}[1][\forewordname]{\chapter*{#1}}
\newcommand{\contributors}[1][\contriblistname]{\chapter*{#1}}");
  DefMacro!("\\extrachap{}", "\\chapter*{#1}");
  DefMacro!("\\Extrachap{}", "\\chapter*{#1}");
  // svmult.cls:441-442: the text on the back of a part's title page, plain text here.
  RawTeX!(r"\newenvironment{partbacktext}{}{}");
  // svmult.cls:1296-1298: a rule twice `\arrayrulewidth` thick, the tables' top and bottom rule — a `\hline`.
  // 1511.08212, 1607.05986.
  Let!("\\svhline", "\\hline");
  // svmult.cls:1731-1733 (svmono.cls:1521-1523): `{prob}`, a problem numbered without a caption (`\nocaption`), and
  // `{sol}{<label>}`, its solution, headed by the problem's number in bold (1509.02295, 1001.3294).
  // With no caption a reference to a problem is its number alone, not the theorem's "<name> <number>".
  RawTeX!(r"\spnewtheorem{prob}{}{\bfseries}{\rmfamily}\def\typerefnum@prob{\theprob}
\newcommand{\probref}[1]{\textbf{\ref{#1}} }
\newenvironment{sol}{\par\noindent\probref}{\par}");
  // svmult.cls:182-208, 822-850 (svmono.cls:183-210, 690-719): the gray and tinted boxes — shaded (framed.sty) under
  // `graybox`; otherwise the class warns and sets a quotation-like list. v5.5/v5.6 (2018, svmult.cls:2430-2441)
  // redefine `{svgraybox}` `\AtBeginDocument` on a `\MakeFramed` `{newshaded}` box whatever the options, which
  // needs framed.sty — loaded only under `graybox` — so there a document without it errors in pdflatex (an undefined
  // `\MakeFramed`); the binding keeps the per-option definitions. 0801.0527, 0805.1759, 0808.3224.
  if lookup_bool("svmult_graybox") {
    RequirePackage!("color");
    RequirePackage!("framed");
    RawTeX!(r"\definecolor{shadecolor}{gray}{.85}\definecolor{tintedcolor}{gray}{.80}
\newenvironment{svgraybox}{\begin{shaded}}{\end{shaded}}
\newenvironment{svtintedbox}{\definecolor{shadecolor}{gray}{.80}\begin{shaded}}{\end{shaded}}");
  } else {
    RawTeX!(r#"\newenvironment{svgraybox}{\ClassWarning{Springer-SVMono}{Environment "svgraybox" not available,\MessageBreak
switching over to "quotation" environment;\MessageBreak specify documentclass option "graybox",\MessageBreak
see SVMono documentation -}\begin{quotation}}{\end{quotation}}
\newenvironment{svtintedbox}{\ClassWarning{Springer-SVMono}{Environment "svtintedbox" not available,\MessageBreak
switching over to "quotation" environment;\MessageBreak specify documentclass option "graybox",\MessageBreak
see SVMono documentation -}\begin{quotation}}{\end{quotation}}"#);
  }

  // svmult.cls:321-322, 538-541, 570-571 (svmono.cls:318-319, 424-427): `\motto[<width>]{<text>}` and `\chapauthor
  // {<names>}` store what the next chapter head sets under its title (`\processchapauthor`, `\processmotto` in
  // `\@makechapterhead`, :634-635; a starred head only the motto, `\@makeschapterhead` :649): the
  // chapter's author a creator of that chapter, the motto a quotation in the class's motto font, read through the
  // kernel's `\lx@startsection@afterhook` inside the new chapter (0911.4644, 1112.2297 `\motto`; 1010.1304, 1009.1542
  // `\chapauthor`).
  RawTeX!(r"\newcommand{\mottosize}{\small}\newcommand{\mottostyle}{\itshape\unboldmath\raggedright}
\newdimen\mottowidth
\newcommand\motto[2][77mm]{\setlength{\mottowidth}{#1}\gdef\m@ttotext{#2}}
\newcommand{\chapauthor}[1]{\gdef\ch@pauthor{#1}}
\def\lx@sv@chapterhead{\@ifundefined{ch@pauthor}{}{\lx@sv@chapauthor{\ch@pauthor}\global\let\ch@pauthor\undefined}%
\lx@sv@schapterhead}
\def\lx@sv@schapterhead{\@ifundefined{m@ttotext}{}{\begin{quote}\mottosize\mottostyle\m@ttotext\end{quote}%
\global\let\m@ttotext\undefined}}");
  DefConstructor!("\\lx@sv@chapauthor{}",
    "<ltx:creator role='author'><ltx:personname>#1</ltx:personname></ltx:creator>");
  DefMacro!("\\lx@startsection@afterhook{}{}", sub[(stype, inlist)] {
    Ok(match (stype.to_string().as_str(), inlist.to_string().is_empty()) {
      ("chapter", false) => Tokens!(T_CS!("\\lx@sv@chapterhead")),
      ("chapter", true) => Tokens!(T_CS!("\\lx@sv@schapterhead")),
      _ => Tokens!(),
    })
  });

  DefMacro!("\\propositionname", "Proposition");

  // TOC registers
  DefRegister!("\\tocchpnum"         => Dimension::new(0));
  DefRegister!("\\tocsecnum"         => Dimension!("18pt"));
  DefRegister!("\\tocsectotal"       => Dimension::new(0));
  DefRegister!("\\tocsubsecnum"      => Dimension::new(0));
  DefRegister!("\\tocsubsectotal"    => Dimension::new(0));
  DefRegister!("\\tocsubsubsecnum"   => Dimension::new(0));
  DefRegister!("\\tocsubsubsectotal" => Dimension::new(0));
  DefRegister!("\\tocparanum"        => Dimension::new(0));
  DefRegister!("\\tocparatotal"      => Dimension::new(0));
  DefRegister!("\\tocsubparanum"     => Dimension::new(0));

  def_macro_noop("\\dominitoc")?;
  def_macro_noop("\\calctocindent")?;

  def_macro_noop("\\clearheadinfo")?;
  def_macro_noop("\\clearemptydoublepage")?;
  // Springer Multi-author authors use \orcid for the ORCID identifier. Route
  // through the kernel `\lx@add@orcid` → `ltx:contact[role=orcid]` → clickable
  // orcid.org link + logo icon (vs a bare dagger note). html_feedback#6571.
  // Witness 2408.17087, 2411.17645 (2 svmult papers).
  DefMacro!("\\orcid{}", "\\lx@add@orcid{#1}");

  // What the 2018+ svmult.cls versions add (2605.18420's copy; 82-117 of 120 sampled copies, so a paper whose copy
  // lacks one keeps its own definition: `DefinitionOrigin::Fallback`). Run 340 first errors: `\orcidID` 80 of 120
  // sampled svmult papers, `\ethics` 25, the boxes 5-9 each, `\biblstarthook` 10 (~350 papers; 2605.18420):
  // - `\orcidID{<id>}` (:2192), an author's ORCID (the class prints it as a superscript), as llncs's;
  // - `\ethics{<title>}{<text>}` and `\ethicssubhead` (:2575-2577), the declarations block, small, its title bold;
  // - the titled boxes `{trailer}`, `{questype}`, `{important}`, `{warning}`, `{programcode}`, `{tips}`, `{overview}`,
  //   `{backgroundinformation}`, `{legaltext}` (:2497-2525, `\formtmp`): a shaded title bar, bold, over the text and a
  //   closing rule; the title bold, the text below (the circled marks of three of them, and the colours, dropped);
  // - `\biblstarthook{<text>}` (:2029), typeset under the bibliography's heading, once: through the kernel's
  //   `\lx@bibliography@preamble`, before what natbib's `\bibpreamble` adds there;
  // - `\subsubruninhead` (:754-758), a bold heading (its positive after-skip sets it on a line of its own); `{petit}` (:334), small text; `{refguide}` (:2555), the
  //   guide's sectioning; `\leftcaption`/`\rightcaption` (:1864-1881), captions of figures set side by side, and
  //   `\leftfigure`/`\rightfigure` (:1888-1889) their contents; `\samenumber` (:2024), the next caption keeps the
  //   number.
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    DefMacro!("\\orcidID{}", "\\lx@add@orcid{#1}");
    RawTeX!(
      r"\def\ethicssubhead#1#2{\par\noindent{\bfseries #1}\enspace#2}
\def\ethics#1#2{\bgroup\small\if!#1!\else\par\noindent{\bfseries#1}\enspace\nobreak\fi{\noindent #2\endgraf}\egroup}
\def\lx@sv@box#1{\par\noindent{\bfseries #1}\par}
\newenvironment{trailer}[1]{\lx@sv@box{#1}}{\par}
\newenvironment{questype}[1]{\lx@sv@box{#1}}{\par}
\newenvironment{important}[1]{\lx@sv@box{#1}}{\par}
\newenvironment{warning}[1]{\lx@sv@box{#1}}{\par}
\newenvironment{programcode}[1]{\lx@sv@box{#1}}{\par}
\newenvironment{tips}[1]{\lx@sv@box{#1}}{\par}
\newenvironment{overview}[1]{\lx@sv@box{#1}}{\par}
\newenvironment{backgroundinformation}[1]{\lx@sv@box{#1}}{\par}
\newenvironment{legaltext}[1]{\lx@sv@box{#1}}{\par}
\def\biblstarthook#1{\gdef\biblst@rthook{#1}}
\AtBeginDocument{\let\lx@sv@bibpreamble\lx@bibliography@preamble
  \def\lx@bibliography@preamble{\csname biblst@rthook\endcsname\global\let\biblst@rthook\undefined\lx@sv@bibpreamble}}
\newcommand\subsubruninhead{\@startsection{paragraph}{4}{\z@}{-6\p@}{0.1\p@}{\normalfont\normalsize\bfseries}}
\newenvironment{petit}{\par\small}{\par}
\newenvironment{refguide}{}{}
\def\leftcaption{\caption}\def\rightcaption{\caption}
\newcommand{\leftfigure}[2][l]{#2}\let\rightfigure\leftfigure
\def\samenumber{\addtocounter{\@captype}{-1}}"
    );
  }
  // What every release from v1.0 (1999) to v5.15 defines (0912.1479's v5.4 copy; 80-99 of the 102 copies papers ship;
  // run 347: 48 svmult papers call one), a copy without one leaving it to the paper (`DefinitionOrigin::Fallback`;
  // Perl's binding defines none of it, KNOWN_PERL_ERRORS #812):
  // - :1285 `\let\ts\,`, a thin space (17 papers: 1001.0767, 1405.1421, cond-mat0605750);
  // - :115 `\if@runhead`, true unless the `norunningheads` option (v1.0 :37-38: false unless `runningheads`), which
  //   papers test (0803.2826, 0808.2157, 0810.2505);
  // - :240-242 `\svparindent` (12pt), :949 `\verbatimindent`, :1756-1758 `\twocaptionwidth{w1}{w2}` (1607.02736,
  //   1804.07860, 1605.01046): layout;
  // - :376 `\mtaddtocont{entry}` and :734 `\hyperhrefextend`, a mini table of contents' line, written to the aux file
  //   (1310.7882, 2311.01874);
  // - :1903-1923 `\subfigures`, after which the figure's captions are numbered as one figure lettered (3a, 3b), the
  //   counter restored at the figure's end (:1905 `\def\endfigure{\resetsubfig\end@float}`; 1310.5064, 1411.2805,
  //   1710.10378; `{subfigures}` around several figures, 2008.12552);
  // - :2307-2310 `{abbrsymblist}[label width]`, the "List of Abbreviations and Symbols" section, a description list
  //   (0810.2505, cond-mat0501076);
  // - :987 `\greeksym{a}`, the letter set in the Symbol font: the Greek letter the font has there, upright (Adobe
  //   Symbol encoding, OXIDIZED_DESIGN #623; 0903.0562, 1111.1183's `\uDelta` `\mbox{\greeksym{D}}`), any other argument in that font;
  // - v1.0 :74 `\micro`, an upright mu (astro-ph0108509, hep-ph0301145);
  // - v5.5 :2392-2402 `\circledmark[colour]{mark}` and `\formtmp{colour}{title}`, the titled boxes' mark and title bar
  //   (their colours dropped, as the boxes' above);
  // - v5.15 :2589-2591 `\Description{text}`, the description of a figure's image, written to a file of them: the
  //   float's description, as acmart's (2602.21993).
  {
    let _class = OriginGuard::new(DefinitionOrigin::Fallback);
    let runhead_default = crate::shipped_class::shipped_class_text("svmult")?
      .is_none_or(|text| !crate::shipped_class::class_code_contains(&text, "\\DeclareOption{runningheads}"));
    RawTeX!(if runhead_default { r"\let\if@runhead\iftrue" } else { r"\let\if@runhead\iffalse" });
    RawTeX!(
      r"\let\ts\,
\newdimen\svparindent
\setlength{\svparindent}{12\p@}
\newdimen\verbatimindent \verbatimindent\parindent
\def\twocaptionwidth#1#2{\def\first@capwidth{#1}\def\second@capwidth{#2}}
\twocaptionwidth{.46\textwidth}{.46\textwidth}
\newcommand{\mtaddtocont}[1]{\protected@write\@auxout{\let\label\@gobble\let\index\@gobble\let\glossary\@gobble}{\string\@mtwritefile{#1}}}
\def\hyperhrefextend{\ifx\hyper@anchor\@undefined\else{\@currentHref}\fi}
\newcounter{merk}
\def\resetsubfig{\global\let\last@subfig=\undefined}
\def\r@setsubfig{\xdef\last@subfig{\number\value{figure}}\setcounter{figure}{\value{merk}}\setcounter{merk}{0}}
\def\subfigures{\refstepcounter{figure}\@tempcnta=\value{merk}\setcounter{merk}{\value{figure}}\setcounter{figure}{\the\@tempcnta}\def\thefigure{\ifnum\c@chapter>\z@\thechapter.\fi\@arabic\c@merk\alph{figure}}\let\resetsubfig=\r@setsubfig}
\AddToHook{env/figure/end}{\resetsubfig}
\AddToHook{env/figure*/end}{\resetsubfig}
\def\abbrsymbname{List of Abbreviations and Symbols}
\newenvironment{abbrsymblist}[1][\qquad]{\section*{\abbrsymbname}\begin{description}}{\end{description}}
\newcommand{\micro}{\mbox{µ}}
\newcommand\circledmark[2][white]{#2}
\def\formtmp#1#2{\lx@sv@box{#2}}"
    );
    DefMacro!("\\greeksym{}", sub[(letter)] {
      let letter: Tokens = letter;
      let text = letter.to_string();
      Ok(match symbol_font_greek(text.trim()) {
        Some(greek) => mouth::tokenize_internal(TeXString::assembled(format!("\\mbox{{\\upshape {greek}}}"))),
        None => {
          let mut tokens = TokenizeInternal!(r"\usefont{U}{psy}{m}{n}").unlist();
          tokens.extend(letter.unlist());
          Tokens::new(tokens)
        },
      })
    });
    crate::float_description::define_float_description("\\Description")?;
    DefMacro!("\\Description{}", sub[(long)] {
      Ok(crate::float_description::float_description_tokens(None, &long))
    });
  }
  // The astronomical journal abbreviations 7 copies paste in (1711.01112, 2002.10370, 1801.09940's svmult_aa.cls).
  crate::shipped_class::provide_shipped_aas_macros("svmult")?;
});

/// The Greek letter the Symbol font (Adobe Symbol encoding, `\usefont{U}{psy}{m}{n}`) has at the ASCII letter
/// `letter`, if it is one.
fn symbol_font_greek(letter: &str) -> Option<char> {
  let mut chars = letter.chars();
  let (Some(c), None) = (chars.next(), chars.next()) else {
    return None;
  };
  const LOWER: &str = "αβχδεφγηιϕκλμνοπθρστυϖωξψζ";
  const UPPER: &str = "ΑΒΧΔΕΦΓΗΙϑΚΛΜΝΟΠΘΡΣΤΥςΩΞΨΖ";
  match c {
    'a'..='z' => LOWER.chars().nth(c as usize - 'a' as usize),
    'A'..='Z' => UPPER.chars().nth(c as usize - 'A' as usize),
    _ => None,
  }
}
