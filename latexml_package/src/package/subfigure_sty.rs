use crate::prelude::*;
use crate::engine::latex_ch9_figures_and_tables::{before_float, after_float};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: subfigure.sty.ltxml

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
\newcommand*{\subcaplabelfont}{%
  \subcaplabelfont@f\subcaplabelfont@c\subcaplabelfont@s}
\newcommand*{\subcaplabelfont@f}{\fontfamily{\familydefault}\selectfont}
\newcommand*{\subcaplabelfont@c}{\fontseries{\seriesdefault}\selectfont}
\newcommand*{\subcaplabelfont@s}{\fontshape{\shapedefault}\selectfont}
\newcommand*{\subcapfont}{%
  \subcapfont@f\subcapfont@c\subcapfont@s}
\newcommand*{\subcapfont@f}{\fontfamily{\familydefault}\selectfont}
\newcommand*{\subcapfont@c}{\fontseries{\seriesdefault}\selectfont}
\newcommand*{\subcapfont@s}{\fontshape{\shapedefault}\selectfont}
\@ifundefined{figuretopcaptrue}{\newif\iffiguretopcap}{}
\newif\ifsubfiguretopcap
\@ifundefined{tabletopcaptrue}{\newif\iftabletopcap}{}
\newif\ifsubtabletopcap
\newif\ifsf@tight          \sf@tighttrue
"""#);

  DeclareOption!("normal",
    "\\subcaphangfalse\\subcapcenterfalse\\subcapcenterlastfalse\\subcapnoonelinefalse\\subcapraggedrightfalse");
  DeclareOption!("hang", "\\subcaphangtrue");
  DeclareOption!("center", "\\subcapcentertrue");
  DeclareOption!("centerlast", "\\subcapcenterlasttrue");
  DeclareOption!("nooneline", "\\subcapnoonelinetrue");
  DeclareOption!("raggedright", "\\subcapraggedrighttrue");
  DeclareOption!("isu", "\\subcaphangtrue");
  DeclareOption!("anne", "\\subcapcenterlasttrue");
  DeclareOption!("scriptsize", "\\renewcommand*{\\subcapsize}{\\scriptsize}");
  DeclareOption!("footnotesize", "\\renewcommand*{\\subcapsize}{\\footnotesize}");
  DeclareOption!("small", "\\renewcommand*{\\subcapsize}{\\small}");
  DeclareOption!("normalsize", "\\renewcommand*{\\subcapsize}{\\normalsize}");
  DeclareOption!("large", "\\renewcommand*{\\subcapsize}{\\large}");
  DeclareOption!("Large", "\\renewcommand*{\\subcapsize}{\\Large}");
  DeclareOption!("rm", "\\renewcommand*{\\subcaplabelfont@f}{\\rmfamily}");
  DeclareOption!("sf", "\\renewcommand*{\\subcaplabelfont@f}{\\sffamily}");
  DeclareOption!("tt", "\\renewcommand*{\\subcaplabelfont@f}{\\ttfamily}");
  DeclareOption!("md", "\\renewcommand*{\\subcaplabelfont@c}{\\mdseries}");
  DeclareOption!("bf", "\\renewcommand*{\\subcaplabelfont@c}{\\bfseries}");
  DeclareOption!("up", "\\renewcommand*{\\subcaplabelfont@s}{\\upshape}");
  DeclareOption!("it", "\\renewcommand*{\\subcaplabelfont@s}{\\itshape}");
  DeclareOption!("sl", "\\renewcommand*{\\subcaplabelfont@s}{\\slshape}");
  DeclareOption!("sc", "\\renewcommand*{\\subcaplabelfont@s}{\\scshape}");
  DeclareOption!("RM", "\\renewcommand*{\\subcapfont@f}{\\rmfamily}");
  DeclareOption!("SF", "\\renewcommand*{\\subcapfont@f}{\\sffamily}");
  DeclareOption!("TT", "\\renewcommand*{\\subcapfont@f}{\\ttfamily}");
  DeclareOption!("MD", "\\renewcommand*{\\subcapfont@c}{\\mdseries}");
  DeclareOption!("BF", "\\renewcommand*{\\subcapfont@c}{\\bfseries}");
  DeclareOption!("IT", "\\renewcommand*{\\subcapfont@s}{\\itshape}");
  DeclareOption!("SL", "\\renewcommand*{\\subcapfont@s}{\\slshape}");
  DeclareOption!("SC", "\\renewcommand*{\\subcapfont@s}{\\scshape}");
  DeclareOption!("UP", "\\renewcommand*{\\subcapfont@s}{\\upshape}");
  DeclareOption!("figbotcap", "\\figuretopcapfalse");
  DeclareOption!("figtopcap", "\\figuretopcaptrue");
  DeclareOption!("tabbotcap", "\\tabletopcapfalse");
  DeclareOption!("tabtopcap", "\\tabletopcaptrue");
  DeclareOption!("FIGBOTCAP", "\\ExecuteOptions{figbotcap}\\subfiguretopcapfalse");
  DeclareOption!("FIGTOPCAP", "\\ExecuteOptions{figtopcap}\\subfiguretopcaptrue");
  DeclareOption!("TABBOTCAP", "\\ExecuteOptions{tabbotcap}\\subtabletopcapfalse");
  DeclareOption!("TABTOPCAP", "\\ExecuteOptions{tabtopcap}\\subtabletopcaptrue");
  DeclareOption!("loose",
    "\\subfigtopskip=10\\p@ \\subfigcapskip=10\\p@ \\subfigcaptopadj=0\\p@ \\subfigbottomskip=10\\p@ \\subfigcapmargin=10\\p@ \\subfiglabelskip=0.33em \\sf@tightfalse");
  DeclareOption!("tight",
    "\\subfigtopskip=5\\p@ \\subfigcapskip=0\\p@ \\subfigcaptopadj=3\\p@ \\subfigbottomskip=5\\p@ \\subfigcapmargin=\\z@ \\subfiglabelskip=0.33em plus 0.07em minus 0.03em \\sf@tighttrue");

  Digest!("\\ExecuteOptions{normal,footnotesize,FIGBOTCAP,TABBOTCAP,loose}")?;
  ProcessOptions!();

  NewCounter!("subfigure", "figure", idprefix => "sf", idwithin => "figure");
  NewCounter!("subtable", "table", idprefix => "st", idwithin => "table");
  DefMacro!("\\thesubfigure", "(\\alph{subfigure})");
  DefMacro!("\\thesubtable", "(\\alph{subtable})");
  Let!("\\p@subfigure", "\\thefigure");
  Let!("\\p@subtable", "\\thetable");
  Let!("\\ext@subfigure", "\\ext@figure");
  Let!("\\ext@subtable", "\\ext@table");

  DefMacro!("\\fnum@font@subfigure", "\\subcapsize\\subcaplabelfont");
  DefMacro!("\\fnum@font@subtable", "\\subcapsize\\subcaplabelfont");
  DefMacro!("\\format@title@font@subfigure", "\\subcapsize\\subcapfont");
  DefMacro!("\\format@title@font@subtable", "\\subcapsize\\subcapfont");

  // \subfigure — Perl: subfigure.sty.ltxml L138-149
  DefMacro!("\\subfigure[][]{}",
    "\\begin{@subfigure}\\iffiguretopcap\\else\\addtocounter{figure}{1}\\fi\\ifsubfiguretopcap\\ifx.#2.\\ifx.#1.\\else\\caption{#1}\\fi\\else\\caption[#1]{#2}\\fi#3\\else#3\\ifx.#2.\\ifx.#1.\\else\\caption{#1}\\fi\\else\\caption[#1]{#2}\\fi\\fi\\iffiguretopcap\\else\\addtocounter{figure}{-1}\\fi\\end{@subfigure}");

  // {@subfigure} environment — Perl: subfigure.sty.ltxml L151-158
  DefEnvironment!("{@subfigure}",
    "^<ltx:figure xml:id='#id'>#tags #body</ltx:figure>",
    mode => "restricted_horizontal",
    before_digest => {
      before_float("subfigure", None);
    },
    after_digest => sub[whatsit] { after_float(whatsit); });

  // \subtable — Perl: subfigure.sty.ltxml L160-171
  DefMacro!("\\subtable[][]{}",
    "\\begin{@subtable}\\iftabletopcap\\else\\addtocounter{table}{1}\\fi\\ifsubtabletopcap\\ifx.#2.\\ifx.#1.\\else\\caption{#1}\\fi\\else\\caption[#1]{#2}\\fi#3\\else#3\\ifx.#2.\\ifx.#1.\\else\\caption{#1}\\fi\\else\\caption[#1]{#2}\\fi\\fi\\iftabletopcap\\else\\addtocounter{table}{-1}\\fi\\end{@subtable}");

  // {@subtable} environment — Perl: subfigure.sty.ltxml L172-179
  DefEnvironment!("{@subtable}",
    "^<ltx:table xml:id='#id'>#tags #body</ltx:table>",
    mode => "restricted_horizontal",
    before_digest => {
      before_float("subtable", None);
    },
    after_digest => sub[whatsit] { after_float(whatsit); });

  // Perl: DefMacro('\subref OptionalMatch:* Semiverbatim', '\ref{#2}');
  // Note: OptionalMatch:* blocked by codegen star bug — use simple form
  DefMacro!("\\subref Semiverbatim", "\\ref{#1}");
});
