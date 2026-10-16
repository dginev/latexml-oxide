//! ccaption.sty — extended captioning by Peter Wilson (Herries Press).
//!
//! ccaption provides:
//!  * Non-numbered/`*` captions (`\bottomcaption`, `\namedlegend`)
//!  * Continued captions (`\contcaption`)
//!  * Bilingual captions (`\bicaption`)
//!  * Sidecap-style captions, custom caption fonts/labels
//!  * `[subfigure]` compatibility: redefines `\caption`, `\@subcaption`, `\@float`, `\@dbflt` to
//!    track main-vs-sub state.
//!
//! Raw-loading ccaption.sty into our engine is fragile: with the
//! `[subfigure]` option, ccaption redefines `\@subcaption` to use
//! `\xdef\@subfigcaptionlist{... {\protect\numberline {\@currentlabel}
//! ...}}`. Our `\@currentlabel` expansion produces tokens like
//! `\the\c@figure` that the `\xdef` mis-handles, surfacing the
//! "You can't use `}` after `\the`" cascade at every figure +
//! `\caption{...}\label{a}` pair (driver: 1105.3285).
//!
//! Perl LaTeXML has no ccaption binding either; with default
//! `INCLUDE_STYLES=false`, raw ccaption.sty is not loaded. Match
//! Perl: stub the user-facing extension macros as thin wrappers
//! over `\caption`, leaving the kernel `\caption`/`\label`/
//! `\@subcaption` untouched.
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "ccaption.sty",
    "ccaption.sty is minimally stubbed — extended caption features (\\bicaption, \\contcaption, \\bottomcaption, sidecap-style) fall back to the base \\caption."
  );
  // ccaption.sty:25-44 the options: `subfigure` (and the older `subfigure20`/`subfigure21`) adapts the subfigure
  // package's subfloats to continued figures (below); `caption2` and `titles` change nothing here.
  RawTeX!(r"\newif\if@contsubfig\@contsubfigfalse");
  DeclareOption!("subfigure", {
    Digest!(r"\@contsubfigtrue")?;
  });
  DeclareOption!("subfigure20", {
    Digest!(r"\@contsubfigtrue")?;
  });
  DeclareOption!("subfigure21", {
    Digest!(r"\@contsubfigtrue")?;
  });
  DeclareOption!("caption2", None);
  DeclareOption!("titles", None);
  ProcessOptions!();
  // ccaption.sty:46-48 and :66-96: the caption-shape commands as the file defines them — the width a caption is set
  // in (`\captionwidth{<len>}`, 24 papers: 0803.1137, 0806.4323, 0811.1937; `\changecaptionwidth`, 20 papers: 0905.4349,
  // 1207.5269, 1501.00230; `\normalcaptionwidth`), `\centerlastline` and `\flushleftright` (paragraph shapes inside
  // a caption, 1706.01843), and `\precaption`/`\postcaption`/`\midbicaption{<code>}`, stored for the caption builder
  // (0711.3402, 1001.1088, 1012.0535, 1408.3003); the captions keep LaTeXML's layout. Perl has no ccaption binding
  // (KNOWN_PERL_ERRORS #807).
  RawTeX!(
    r"\newif\if@contcw\newif\if@conthang\newif\if@contindent
\newcommand{\flushleftright}{\leftskip\z@ \rightskip\z@ \parfillskip=\z@ plus 1fil}
\newcommand{\centerlastline}{\leftskip=\z@ plus 1fil \rightskip=\z@ plus -1fil \parfillskip=\z@ plus 2fil}
\newlength{\@contcwidth}
\newcommand{\captionwidth}[1]{\setlength{\@contcwidth}{#1}}
\captionwidth{\linewidth}
\newcommand{\changecaptionwidth}{\@contcwtrue}
\newcommand{\normalcaptionwidth}{\@contcwfalse}
\normalcaptionwidth
\newcommand{\precaption}[1]{\def\@contpre{#1}}
\precaption{}
\newcommand{\postcaption}[1]{\def\@contpost{#1}}
\postcaption{}
\newcommand{\midbicaption}[1]{\def\@contmidbi{#1}}
\midbicaption{}"
  );
  // ccaption.sty:204-217 `\bitwonumcaption[<label>]{<lof>}{<caption>}{<name>}{<lof2>}{<caption2>}`: a bilingual caption,
  // each language numbered — the second caption under the float name `<name>`, the counter stepped back between them
  // (1302.5179: "Figure 1: …" then "Tableau 1: …"); :219-259 its siblings with one number, `\bionenumcaption`,
  // `\bicaption[<label>]{<lof>}{<caption>}{<name>}{<caption2>}` (the binding's former four-argument `\bicaption` kept
  // only the first caption and left the rest as text) and `\bicontcaption`, the second caption a `\contcaption`.
  // `\@if@contemptyarg` (:199-202) tells an empty list entry.
  RawTeX!(
    r"\begingroup
\catcode`\Q=3
\long\gdef\@if@contemptyarg#1{\@xif@contmt#1QQ\@secondoftwo\@firstoftwo\@nil}
\long\gdef\@xif@contmt#1#2Q#3#4#5\@nil{#4}
\endgroup
\newcommand{\bitwonumcaption}[6][\@empty]{%
  \begingroup
  \@if@contemptyarg{#2}{\caption{#3}}{\caption[#2]{#3}}%
  \ifx\@empty#1\else\label{#1}\fi
  \edef\@conttempc{#4}%
  \expandafter\renewcommand\csname \@captype name\endcsname{\@conttempc}%
  \addtocounter{\@captype}{-1}%
  \@contmidbi
  \@if@contemptyarg{#5}{\caption{#6}}{\caption[#5]{#6}}%
  \endgroup}
\newcommand{\bionenumcaption}[6][\@empty]{%
  \begingroup
  \@if@contemptyarg{#2}{\caption{#3}}{\caption[#2]{#3}}%
  \ifx\@empty#1\else\label{#1}\fi
  \edef\@conttempc{#4}%
  \expandafter\renewcommand\csname \@captype name\endcsname{\@conttempc}%
  \@contmidbi
  \contcaption{#6}%
  \endgroup}
\newcommand{\bicaption}[5][\@empty]{%
  \begingroup
  \@if@contemptyarg{#2}{\caption{#3}}{\caption[#2]{#3}}%
  \ifx\@empty#1\else\label{#1}\fi
  \edef\@conttempc{#4}%
  \expandafter\renewcommand\csname \@captype name\endcsname{\@conttempc}%
  \@contmidbi
  \contcaption{#5}%
  \endgroup}
\newcommand{\bicontcaption}[3]{%
  \begingroup
  \contcaption{#1}%
  \edef\@conttempc{#2}%
  \expandafter\renewcommand\csname \@captype name\endcsname{\@conttempc}%
  \@contmidbi
  \contcaption{#3}%
  \endgroup}"
  );
  // ccaption.sty:309-312 and :382-430 under `subfigure`: a continued figure's subfloats go on counting from the
  // previous float's (`\contsubbottom`/`\contsubtop`, and `\contsubfigure`/`\contsubtable` for them: 0706.2285,
  // 0907.2099, 0907.2405), the count kept after every subfloat (`\@contkeep`, through the subfigure binding's
  // `\lx@subfloat@done`) and restored before a continued one (`\@contset`); the continued subfloat is numbered within
  // the float it continues, so the figure step a bottom-captioned subfloat anticipates (subfigure_sty.rs) is not
  // taken (`\lx@cc@contsub@step`, defined outside the option's conditional: its `\iftrue` would unbalance the skipped
  // text). `\subconcluded` ends the count; `\subbottom`/`\subtop` are the subfigure package's subfloat of the float
  // type.
  RawTeX!(
    r"\def\lx@cc@contsub@step#1{\expandafter\ifx\csname if\@captype topcap\endcsname\iftrue\else
    \addtocounter{\@captype}{#1}\fi}"
  );
  RawTeX!(
    r"\if@contsubfig
  \newcounter{@contsubnum}
  \newcommand{\@contkeep}{\setcounter{@contsubnum}{\value{sub\@captype}}}
  \newcommand{\@contset}{\setcounter{sub\@captype}{\value{@contsubnum}}}
  \newcommand{\subconcluded}{\setcounter{sub\@captype}{0}}
  \def\lx@subfloat@done{\@contkeep}
  \newcommand{\subbottom}{\csname sub\@captype\endcsname}
  \newcommand{\subtop}{\csname sub\@captype\endcsname}
  \newcommand{\contsubbottom}[2][]{\@contset\lx@cc@contsub@step{-1}%
    \csname sub\@captype\endcsname[#1]{#2}\lx@cc@contsub@step{1}}
  \let\contsubtop\contsubbottom
  \let\contsubfigure\contsubbottom
  \let\contsubtable\contsubbottom
\fi"
  );
  // ccaption.sty:138-141 `\contcaption{<text>}`: the caption of a continued float, under the number of the float it
  // continues (the counter stepped back before it is stepped; 0907.2405 "Figure 1: Continued."). Split and bilingual
  // captions otherwise fall back to the regular caption.
  DefMacro!(
    "\\contcaption{}",
    "\\addtocounter{\\@captype}{\\m@ne}\\caption{#1}"
  );
  DefMacro!("\\bottomcaption{}", "\\caption{#1}");
  DefMacro!("\\legend{}", "\\caption{#1}");
  DefMacro!("\\namedlegend{}", "\\caption{#1}");
  // Sub-captions (when ccaption's `[subfigure]` compat would be
  // active): forward to the standalone subfigure/subcaption packages
  // when available; otherwise emit as italic text. Both `\subcaption`
  // and `\subtop`/`\subbottom` follow this pattern.
  DefMacro!("\\subcaption{}", "{\\itshape #1}");
  RawTeX!(
    r"\@ifundefined{subtop}{\def\subtop#1{{\itshape #1}}}{}\@ifundefined{subbottom}{\def\subbottom#1{{\itshape #1}}}{}"
  );
  // Caption-formatting / -setup commands ccaption provides — accept
  // and discard.
  DefMacro!("\\captionnamefont{}", "");
  DefMacro!("\\captiontitlefont{}", "");
  DefMacro!("\\captiondelim{}", "");
  DefMacro!("\\captionstyle{}", "");
  DefMacro!("\\hangcaption", "\\@conthangtrue\\@contindentfalse");
  DefMacro!("\\indentcaption{}", "\\@conthangfalse\\@contindenttrue");
  DefMacro!("\\normalcaption", "\\@conthangfalse\\@contindentfalse");
  // `\newfixedcaption{cs}{type}` defines `cs` as a `\caption` for a
  // specific float type. Define `cs` to forward.
  DefMacro!("\\newfixedcaption[]{}{}", "\\providecommand#2{\\caption}");
  DefMacro!("\\renewfixedcaption[]{}{}", "\\renewcommand#2{\\caption}");
  DefMacro!(
    "\\providefixedcaption[]{}{}",
    "\\providecommand#2{\\caption}"
  );
  // Continued-figure subitem counter (ccaption tracks via
  // `\@captype`); user code may `\thesubfigure` etc. We provide a
  // benign `\thesubfigure` placeholder if not defined.
  def_macro_noop("\\contlabel{}")?;
  def_macro_noop("\\bicaptionbox{}{}")?;
});
