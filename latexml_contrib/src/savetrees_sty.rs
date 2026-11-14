use latexml_package::prelude::*;

LoadDefinitions!({
  // savetrees.sty:181 reads its options with `\ProcessOptionsX`, which marks them processed: `\@curroptions` stays as it was.
  key_options_processed()?;
  // savetrees.sty:32-34.
  RequirePackage!("xkeyval");
  RequirePackage!("ifpdf");
  RequirePackage!("ifluatex");
  // savetrees.sty:26-31 and :49-181, verbatim: the option values, one `\if@st@tight@<feature>`
  // switch per feature (tight by default) set by `<feature>=tight|normal`, the `all`, `subtle`,
  // `moderate` and `extreme` presets, and the obsolete `normal<feature>` options. The binding
  // reads the switches whose features it models: the packages loaded, and `\savetreesbibnote`.
  RawTeX!(
    r"\newcommand*{\st@margin@width}{1.5cm}
\newcommand*{\st@parindent}{1em}
\newcommand*{\st@baselinestretch}{0.95}
\newcommand*{\st@char@shrink}{50}
\newcommand*{\st@cspace@shrink}{-25}
\newcommand*{\st@wspace@factor}{0.8}
\newcommand*{\st@define@option}[1]{%
  \expandafter\newif\csname if@st@tight@#1\endcsname
  \csname @st@tight@#1true\endcsname
  \define@choicekey{savetrees}{#1}[\st@arg\st@arg@num]{tight,normal}[tight]{%
    \ifnum\st@arg@num=0
      \csname @st@tight@#1true\endcsname
    \else
        \csname @st@tight@#1false\endcsname
    \fi
  }%
  \DeclareOptionX{#1}[tight]{\csname KV@savetrees@#1\endcsname{##1}}%
}
\st@define@option{sections}
\st@define@option{margins}
\st@define@option{lists}
\st@define@option{floats}
\st@define@option{indent}
\st@define@option{title}
\st@define@option{leading}
\st@define@option{paragraphs}
\st@define@option{charwidths}
\st@define@option{tracking}
\st@define@option{wordspacing}
\st@define@option{bibliography}
\st@define@option{bibnotes}
\st@define@option{bibbreaks}
\st@define@option{mathspacing}
\st@define@option{mathdisplays}
\define@choicekey{savetrees}{all}[\st@arg\st@arg@num]{tight,normal}[tight]{%
  \ifnum\st@arg@num=0
    \@st@tight@sectionstrue
    \@st@tight@marginstrue
    \@st@tight@liststrue
    \@st@tight@floatstrue
    \@st@tight@indenttrue
    \@st@tight@titletrue
    \@st@tight@leadingtrue
    \@st@tight@paragraphstrue
    \@st@tight@charwidthstrue
    \@st@tight@trackingtrue
    \@st@tight@wordspacingtrue
    \@st@tight@bibliographytrue
    \@st@tight@bibnotestrue
    \@st@tight@bibbreakstrue
    \@st@tight@mathspacingtrue
    \@st@tight@mathdisplaystrue
  \else
    \@st@tight@sectionsfalse
    \@st@tight@marginsfalse
    \@st@tight@listsfalse
    \@st@tight@floatsfalse
    \@st@tight@indentfalse
    \@st@tight@titlefalse
    \@st@tight@leadingfalse
    \@st@tight@paragraphsfalse
    \@st@tight@charwidthsfalse
    \@st@tight@trackingfalse
    \@st@tight@wordspacingfalse
    \@st@tight@bibliographyfalse
    \@st@tight@bibnotesfalse
    \@st@tight@bibbreaksfalse
    \@st@tight@mathspacingfalse
    \@st@tight@mathdisplaysfalse
  \fi
}
\DeclareOptionX{all}[tight]{\KV@savetrees@all{#1}}
\define@key{savetrees}{subtle}{%
  \setkeys{savetrees}{%
    all=normal,
    paragraphs=tight,
    floats=tight,
    mathspacing=tight,
    wordspacing=tight,
    tracking=tight,
    bibbreaks=tight
  }%
}
\DeclareOptionX{subtle}{\KV@savetrees@subtle}
\define@key{savetrees}{moderate}{%
  \setkeys{savetrees}{%
    subtle=yes,
    charwidths=tight,
    mathdisplays=tight,
    lists=tight,
    indent=tight,
    leading=tight,
    bibnotes=tight
  }%
}
\DeclareOptionX{moderate}{\KV@savetrees@moderate}
\DeclareOptionX{extreme}{%
  \setkeys{savetrees}{all=tight}%
}
\DeclareOptionX{marginwidth}{\gdef\st@margin@width{#1}}
\DeclareOptionX{parindent}{\gdef\st@parindent{#1}}
\DeclareOptionX{leadingfraction}{\gdef\st@baselinestretch{#1}}
\DeclareOptionX{charwidthfraction}{%
  \@tempdima=#1pt
  \multiply\@tempdima by -1000
  \advance\@tempdima by 1000pt
  \divide\@tempdima by 65536
  \@tempcnta=\@tempdima
  \xdef\st@char@shrink{\the\@tempcnta}%
}
\DeclareOptionX{trackingfraction}{%
  \@tempdima=#1pt
  \advance\@tempdima by -1pt
  \multiply\@tempdima by 1000
  \divide\@tempdima by 65536
  \@tempcnta=\@tempdima
  \xdef\st@cspace@shrink{\the\@tempcnta}%
}
\DeclareOptionX{wordspacingfraction}{\gdef\st@wspace@factor{#1}}
\newcommand*{\st@mark@as@obsolete}[2]{%
  \define@key{savetrees}{#1}[tight]{%
    \PackageError{savetrees}{Package option `#1' is no longer supported}{%
      Rather than `#1', please specify `#2=normal'.\MessageBreak
      Instead of enabling all features by default and letting the\MessageBreak
      user selectively disable them, savetrees now provides the\MessageBreak
      ability to turn features on or off as desired, including all\MessageBreak
      features en masse.}%
    \csname @st@tight@#2false\endcsname
  }%
  \DeclareOptionX{#1}[tight]{\csname KV@savetrees@#1\endcsname{##1}}%
}
\st@mark@as@obsolete{normalsections}{sections}
\st@mark@as@obsolete{normalmargins}{margins}
\st@mark@as@obsolete{normallists}{lists}
\st@mark@as@obsolete{normalfloats}{floats}
\st@mark@as@obsolete{normalindent}{indent}
\st@mark@as@obsolete{normaltitle}{title}
\st@mark@as@obsolete{normalleading}{leading}
\st@mark@as@obsolete{normallooseness}{paragraphs}
\st@mark@as@obsolete{normalcharwidths}{charwidths}
\st@mark@as@obsolete{normalbib}{bibliography}
\st@mark@as@obsolete{normalbibnotes}{bibnotes}
\ProcessOptionsX\relax"
  );
  // calc and microtype as the switches load them (savetrees.sty:193-194, :286-298): a document
  // can rely on them (calc: OXIDIZED_DESIGN #317); their options are layout, which LaTeXML does
  // not set. titlesec `[tiny,compact]` (`sections`, :182-184) and geometry (`margins`, :185-192)
  // are not loaded: their options restyle the headings and the page, and a default-options
  // document that writes `\titleformat` without loading titlesec itself is not served (open).
  RawTeX!(
    r"\if@st@tight@lists\RequirePackage{calc}\fi
\if@st@tight@charwidths\ifpdf\RequirePackage{microtype}\fi\fi
\if@st@tight@tracking\ifpdf\ifluatex\else\RequirePackage{microtype}\fi\fi\fi"
  );
  DefMacro!("\\bibfont", "\\normalfont\\small");
  def_macro_noop("\\bibsetup")?;
  def_macro_noop("\\markeverypar")?;
  // savetrees.sty:342-346: under `bibnotes=tight` (the default; `moderate` and `extreme` set it,
  // `subtle` clears it) the note and the token after it are dropped.
  RawTeX!(
    r"\if@st@tight@bibnotes
  \newcommand{\savetreesbibnote}[1]{\@gobble}
\else
  \newcommand{\savetreesbibnote}[1]{#1}
\fi"
  );
});
