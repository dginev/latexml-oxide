use crate::prelude::*;

LoadDefinitions!({
  // Minimal stub for subfigure.sty to prevent loading raw TeX from system
  // Full Perl version uses beforeFloat/afterFloat which aren't ported
  TeX!(r#"""
\newif\ifsubcaphang
\newif\ifsubcapcenter
\newif\ifsubcapcenterlast
\newif\ifsubcapnooneline
\newif\ifsubcapraggedright
\newskip\subfigtopskip     \subfigtopskip    =  5\p@
\newskip\subfigcapskip     \subfigcapskip    =  0\p@
\newdimen\subfigcaptopadj  \subfigcaptopadj  =  3\p@
\newskip\subfigbottomskip  \subfigbottomskip =  5\p@
\newdimen\subfigcapmargin  \subfigcapmargin  =  \z@
\newskip\subfiglabelskip   \subfiglabelskip  =  0.33em plus 0.07em minus 0.03em
\newcommand*{\subcapsize}{}
\newcommand*{\subcaplabelfont}{}
\newcommand*{\subcapfont}{}
\@ifundefined{figuretopcaptrue}{\newif\iffiguretopcap}{}
\newif\ifsubfiguretopcap
\@ifundefined{tabletopcaptrue}{\newif\iftabletopcap}{}
\newif\ifsubtabletopcap
\newif\ifsf@tight          \sf@tighttrue
"""#);

  DeclareOption!("normal", "");
  DeclareOption!("hang", "\\subcaphangtrue");
  DeclareOption!("center", "\\subcapcentertrue");
  DeclareOption!("centerlast", "\\subcapcenterlasttrue");
  DeclareOption!("nooneline", "\\subcapnoonelinetrue");
  DeclareOption!("raggedright", "\\subcapraggedrighttrue");
  DeclareOption!("isu", "");
  DeclareOption!("anne", "");
  DeclareOption!("scriptsize", "");
  DeclareOption!("footnotesize", "");
  DeclareOption!("small", "");
  DeclareOption!("normalsize", "");
  DeclareOption!("large", "");
  DeclareOption!("Large", "");
  DeclareOption!("rm", "");
  DeclareOption!("sf", "");
  DeclareOption!("tt", "");
  DeclareOption!("md", "");
  DeclareOption!("bf", "");
  DeclareOption!("up", "");
  DeclareOption!("it", "");
  DeclareOption!("sl", "");
  DeclareOption!("sc", "");
  DeclareOption!("RM", "");
  DeclareOption!("SF", "");
  DeclareOption!("TT", "");
  DeclareOption!("MD", "");
  DeclareOption!("BF", "");
  DeclareOption!("IT", "");
  DeclareOption!("SL", "");
  DeclareOption!("SC", "");
  DeclareOption!("UP", "");
  DeclareOption!("figbotcap", "\\figuretopcapfalse");
  DeclareOption!("figtopcap", "\\figuretopcaptrue");
  DeclareOption!("tabbotcap", "\\tabletopcapfalse");
  DeclareOption!("tabtopcap", "\\tabletopcaptrue");
  DeclareOption!("FIGBOTCAP", "\\figuretopcapfalse\\subfiguretopcapfalse");
  DeclareOption!("FIGTOPCAP", "\\figuretopcaptrue\\subfiguretopcaptrue");
  DeclareOption!("TABBOTCAP", "\\tabletopcapfalse\\subtabletopcapfalse");
  DeclareOption!("TABTOPCAP", "\\tabletopcaptrue\\subtabletopcaptrue");
  DeclareOption!("loose", "");
  DeclareOption!("tight", "");
  ProcessOptions!();

  NewCounter!("subfigure", "figure");
  NewCounter!("subtable", "table");
  DefMacro!("\\thesubfigure", None, "(\\alph{subfigure})");
  DefMacro!("\\thesubtable", None, "(\\alph{subtable})");
  Let!("\\p@subfigure", "\\thefigure");
  Let!("\\p@subtable", "\\thetable");
  Let!("\\ext@subfigure", "\\ext@figure");
  Let!("\\ext@subtable", "\\ext@table");

  DefMacro!("\\fnum@font@subfigure", "");
  DefMacro!("\\fnum@font@subtable", "");
  DefMacro!("\\format@title@font@subfigure", "");
  DefMacro!("\\format@title@font@subtable", "");

  // Simplified: just pass content through
  DefMacro!("\\subfigure[][]{}", "#3");
  DefMacro!("\\subtable[][]{}", "#3");
  DefMacro!("\\subref OptionalMatch:* Semiverbatim", "\\ref{#2}");
});
