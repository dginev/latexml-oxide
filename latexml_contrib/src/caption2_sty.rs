//! caption2.sty — the obsolete caption package (v2.2, a caption3 front end; TL
//! `tex/latex/caption/caption2.sty`), still loaded directly and by journal classes
//! (w-art.cls, SCGE/scpma, POWFI, rmf-d).
//!
//! Without a binding of its own the name falls back to the caption binding by its
//! glued version suffix (Perl Package.pm:2180, `find_file_fallback`), which lacks
//! caption2's interface: `\captionstyle{center}` was undefined (843 papers in arXiv
//! run 336, Perl the same). Registered by its exact name, this binding outranks that
//! fallback, requires the caption binding (which renders the captions) and defines
//! caption2's commands, so a document's settings are taken. The label delimiter and
//! separator are printed text and close the caption's tag; the other settings are layout
//! — the caption style, margins, one-line centering, label font — so the commands store
//! what they set and the styles they name never run. A document's `\captionlabeldelim` reaches
//! the tag's `close` as Perl's `toAttribute` renders it (a skip is its spaces), so a `\kern`
//! delimiter shows as `\kern…` text there (Perl 0.8.8 the same); `\captionlabelfalse`
//! (caption2.sty:144-149) is not honoured — the label is kept.
//! Witnesses math0601389, 1307.4815, 1911.03646, 2507.04618.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("caption");
  // caption2.sty:46-48, :57-127: its parameters and the caption-style registry. caption2's
  // `\ifcaptionlabel`/`\ifonelinecaptions` read caption3's star and slc switches (:67-81);
  // the switches are plain `\newif`s here, the caption binding keeping no such state.
  RawTeX!(r"\newcommand*\captiontwo@Error[2][]{\PackageError{caption2}{#2}{#1}}
\providecommand*\captionlabeldelim{}
\providecommand*\captionlabelsep{}
\providecommand*\captionlabelfont{}
\newcommand*\ifcaptionwidth{\ifdim\captionwidth>\z@}
\newif\ifcaptionlabel \captionlabeltrue
\newif\ifonelinecaptions
\newif\ifignoreLTcapwidth
\providecommand*\setcaptionmargin{\setlength\captionwidth\z@\setlength\captionmargin}
\providecommand*\setcaptionwidth{\setlength\captionmargin\z@\setlength\captionwidth}
\newcommand*\normalcaptionparams{%
\let\captionsize\@empty
\renewcommand*\captionfont{\captionsize}%
\let\captionlabelfont\@empty
\renewcommand*\captionlabeldelim{:}%
\renewcommand*\captionlabelsep{\space}%
\setcaptionmargin\z@
\setlength\captionindent\z@
\onelinecaptionstrue}
\newcommand*\defcaptionstyle[1]{\@namedef{caption@@#1}}
\newcommand*\newcaptionstyle[1]{%
\expandafter\ifx\csname caption@@#1\endcsname\relax
\expandafter\defcaptionstyle
\else
\captiontwo@Error{Caption style `#1' already defined}%
\expandafter\@gobbletwo
\fi
{#1}}
\newcommand*\renewcaptionstyle[1]{%
\expandafter\ifx\csname caption@@#1\endcsname\relax
\captiontwo@Error{Caption style `#1' undefined}%
\expandafter\@gobbletwo
\else
\expandafter\defcaptionstyle
\fi
{#1}}
\newcommand*\dummycaptionstyle[2]{\defcaptionstyle{#1}{#2\usecaptionstyle\caption@style}}
\newcommand*\captionstyle[1]{%
\expandafter\ifx\csname caption@@#1\endcsname\relax
\captiontwo@Error{Undefined caption style `#1'}%
\else
\def\caption@style{#1}%
\fi}
\newcommand*\usecaptionstyle[1]{%
\@ifundefined{caption@@#1}{\captiontwo@Error{Caption style `#1' undefined}}{\@nameuse{caption@@#1}}}");
  // caption2.sty:147 `{\captionlabelfont\captionlabel\captionlabeldelim}\captionlabelsep`: the delimiter and the
  // separator follow the label in the printed caption (`Figure 1. `, `Figure 1~~~`), read when the caption is made.
  // Here they close the caption's tag, the label the kernel's `\format@title@<type>` builds (sect09.rs). The tag's
  // close is attribute text, so the separator — `\space`, or `\enskip`/`\quad`/`\hskip` in classes (w-art.cls) —
  // is one space there, none when the document empties it. The delimiter is a group of its own, as at :147, so a
  // skip ending it (`.\hskip5pt`) does not scan on into the separator.
  RawTeX!(r"\def\lx@captiontwo@close{{\captionlabeldelim}\ifx\captionlabelsep\@empty\else\space\fi}");
  DefMacro!("\\format@title@figure{}", "\\lx@tag[][\\lx@captiontwo@close]{\\lx@fnum@@{figure}}#1");
  DefMacro!("\\format@title@table{}", "\\lx@tag[][\\lx@captiontwo@close]{\\lx@fnum@@{table}}#1");
  // caption2.sty:133-142: the predefined styles, each a caption3 layout (`\caption@make{<layout>}`),
  // which is presentation: named so `\captionstyle`/`\renewcaptionstyle` find them, never typeset.
  RawTeX!(r"\providecommand*\caption@make[1]{}
\newcaptionstyle{normal}{\caption@make{normal}}
\newcaptionstyle{center}{\caption@make{center}}
\newcaptionstyle{centerlast}{\caption@make{centerlast}}
\newcaptionstyle{flushleft}{\caption@make{flushleft}}
\newcaptionstyle{flushright}{\caption@make{flushright}}
\newcaptionstyle{hang}{\caption@make{hang}}
\newcaptionstyle{hang+center}{\caption@make{hang@center}}
\newcaptionstyle{hang+centerlast}{\caption@make{hang@centerlast}}
\newcaptionstyle{hang+flushleft}{\caption@make{hang@flushleft}}
\newcaptionstyle{indent}{\caption@make{indent}}
\newdimen\captionlinewidth
\newdimen\realcaptionwidth
\newcommand*\usecaptionmargin{}
\newcommand\onelinecaption[2]{#2}");
  // caption2.sty:353-372, :431-436: the subfigure caption styles (defined when subfigure is
  // loaded; layout as above).
  RawTeX!(r"\providecommand*\subcapstyle[1]{%
\expandafter\ifx\csname caption@@#1\endcsname\relax
\captiontwo@Error{Undefined caption style `#1'}%
\else
\def\caption@substyle{#1}%
\fi}
\providecommand*\setsubcapstyle{}");

  // caption2.sty:179-224: the options select a style, a caption size or label font, or which
  // packages caption2 adapts (float, longtable, subfigure — layout of their captions).
  DeclareOption!("normal", "\\captionstyle{normal}");
  DeclareOption!("center", "\\captionstyle{center}");
  DeclareOption!("centerlast", "\\captionstyle{centerlast}");
  DeclareOption!("flushleft", "\\captionstyle{flushleft}");
  DeclareOption!("flushright", "\\captionstyle{flushright}");
  DeclareOption!("hang", "\\captionstyle{hang}");
  DeclareOption!("hang+center", "\\captionstyle{hang+center}");
  DeclareOption!("hang+centerlast", "\\captionstyle{hang+centerlast}");
  DeclareOption!("hang+flushleft", "\\captionstyle{hang+flushleft}");
  DeclareOption!("isu", "\\ExecuteOptions{hang}");
  DeclareOption!("indent", "\\captionstyle{indent}");
  DeclareOption!("anne", "\\ExecuteOptions{centerlast}");
  DeclareOption!("scriptsize", "\\g@addto@macro\\captionsize\\scriptsize");
  DeclareOption!("footnotesize", "\\g@addto@macro\\captionsize\\footnotesize");
  DeclareOption!("small", "\\g@addto@macro\\captionsize\\small");
  DeclareOption!("normalsize", "\\g@addto@macro\\captionsize\\normalsize");
  DeclareOption!("large", "\\g@addto@macro\\captionsize\\large");
  DeclareOption!("Large", "\\g@addto@macro\\captionsize\\Large");
  DeclareOption!("up", "\\g@addto@macro\\captionlabelfont\\upshape");
  DeclareOption!("it", "\\g@addto@macro\\captionlabelfont\\itshape");
  DeclareOption!("sl", "\\g@addto@macro\\captionlabelfont\\slshape");
  DeclareOption!("sc", "\\g@addto@macro\\captionlabelfont\\scshape");
  DeclareOption!("md", "\\g@addto@macro\\captionlabelfont\\mdseries");
  DeclareOption!("bf", "\\g@addto@macro\\captionlabelfont\\bfseries");
  DeclareOption!("rm", "\\g@addto@macro\\captionlabelfont\\rmfamily");
  DeclareOption!("sf", "\\g@addto@macro\\captionlabelfont\\sffamily");
  DeclareOption!("tt", "\\g@addto@macro\\captionlabelfont\\ttfamily");
  DeclareOption!("oneline", "\\onelinecaptionstrue");
  DeclareOption!("nooneline", "\\onelinecaptionsfalse");
  DeclareOption!("float", "");
  DeclareOption!("longtable", "");
  DeclareOption!("subfigure", "");
  DeclareOption!("none", "");
  DeclareOption!("all", "");
  DeclareOption!("ruled", "");
  DeclareOption!("boxed", "");
  DeclareOption!("ignoreLTcapwidth", "\\ignoreLTcapwidthtrue");
  // caption2.sty:226-229 `\normalcaptionparams \ExecuteOptions{none,normal} \ProcessOptions*`.
  RawTeX!(r"\normalcaptionparams");
  execute_options(&["none", "normal"])?;
  ProcessOptions!(*);
});
