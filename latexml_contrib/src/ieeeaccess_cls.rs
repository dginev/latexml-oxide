//! Stub for IEEE Access journal class (ieeeaccess.cls).
//!
//! ieeeaccess.cls is a derivative of IEEEtran. Route to the IEEEtran
//! binding and stub the IEEE Access-specific frontmatter macros.
use latexml_package::prelude::*;

use crate::shipped_class::{class_option_given, shipped_class_loads_or};

LoadDefinitions!({
  LoadClass!("IEEEtran");
  RequirePackage!("ifpdf");
  // ieeeaccess.cls:121-131 (1901.05553's copy; 241, 251 in others): `\RequirePackage{color}`, then, unless the
  // `jtehm` option (:36), `\RequirePackage{spotcolor}` for the PANTONE blue, whose `\NewSpotColorSpace`/`\AddSpotColor`/
  // `\SetPageColorSpace` lines the template's papers copy into their preamble (72 papers of run 353 with them
  // undefined, 48 with `\textcolor`/`\definecolor`: 1901.05553, 1907.00310, 1712.00216, 1902.03817). spotcolor has no
  // binding and loads raw; a copy that dropped it (1 of 60 sampled) loads none.
  RequirePackage!("color");
  let jtehm = class_option_given("jtehm")?;
  if !jtehm && shipped_class_loads_or("ieeeaccess", "spotcolor", true)? {
    RequirePackage!("spotcolor");
  }
  // The class's colours (ieeeaccess.cls:121-133): `accessblue` is CMYK .24,1,1,0 under `jtehm`, else the PANTONE 3015 C
  // spot colour `\definecolor{accessblue}{spotcolor}{PANTONE3015C,1.0}`, whose alternate is CMYK 1 0.3 0 0.2 (the
  // `\AddSpotColor` line). color has no `spotcolor` model here ("Unrecognized color model"), so the alternate stands in:
  // the blue a viewer without the spot ink shows. Papers name it in their caption setup
  // (`\DeclareCaptionFont{ieeeblue}{\color{accessblue}}` + `labelfont={bf,ieeeblue}`: 2405.14505, 2205.15416 — an Error
  // per caption once caption's keys took effect, 68h; pdflatex 0). The class's `black` (CMYK 0,0,0,1) is left out: it
  // would turn every black run into a CMYK colour change.
  if jtehm {
    RawTeX!(r"\definecolor{accessblue}{cmyk}{.24,1,1,0}\definecolor{greycolor}{cmyk}{0,0,0,1}");
  } else {
    RawTeX!(r"\definecolor{accessblue}{cmyk}{1,0.3,0,0.2}\definecolor{greycolor}{cmyk}{0,0,0,.8}");
  }
  RawTeX!(r"\definecolor{grey}{cmyk}{0,0,0,.1}");
  // The caption fonts (ieeeaccess.cls:75-79), which papers call in their own caption formats
  // (`\DeclareCaptionLabelFormat{myformat}{\raggedright\figcapfont{\textbf{#1}\textbf{#2}}}`, 2405.14505).
  RawTeX!(
    r"\def\figcapheadfont{\sffamily\fontencoding{T1}\fontseries{b}\fontsize{7}{8.4}\selectfont}%
\def\figcapfont{\def\textit##1{{\sffamily\selectfont\itshape##1}}\sffamily\fontencoding{T1}\fontseries{m}\fontsize{7}{8.4}\selectfont}%
\def\tablecapheadfont{\sffamily\fontencoding{T1}\fontseries{b}\fontsize{7}{8.4}\selectfont}%
\def\tableheadfont{\sffamily\fontencoding{T1}\fontseries{m}\fontsize{7}{8.4}\selectfont}%
\def\tablecapfont{\def\textit##1{{\sffamily\selectfont\itshape##1}}\sffamily\fontencoding{T1}\fontseries{m}\fontsize{7}{8.4}\selectfont}%"
  );

  // IEEE Access frontmatter (ieeeaccess.cls L270, L448-453, L839) —
  // preserve author-supplied content as ltx:note frontmatter.
  DefMacro!(
    "\\history{}",
    "\\@add@frontmatter{ltx:note}[role=history]{#1}"
  );
  DefMacro!(
    "\\corresp{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}"
  );
  def_macro_noop("\\EOD")?;
  // `\Figure[pos](keys)[gopts]{file}{caption}` (ieeeaccess.cls:270-300): a `{figure}` (`{figure*}` when the image is wider
  // than the column — page layout, not kept here) holding `\includegraphics[gopts]{file}` and `\caption{caption}`; the
  // `figkeys` (`topskip`/`midskip`/`botskip`, :262-267) are vertical spacing. As `\@gobble` it dropped the figure and
  // spilled the file name, whose `_` erred (1803.04935, 1910.07402). Guard:
  // `perfect_kernel_batch73a::ieeeaccess_figure_is_a_figure`.
  RawTeX!(
    r"\def\Figure{\@ifnextchar[{\@Figure}{\@Figure[t!]}}
\def\@Figure[#1]{\@ifnextchar({\@@Figure[#1]}{\@@Figure[#1](topskip=0pt)}}
\def\@@Figure[#1](#2){\@ifnextchar[{\@@@Figure[#1](#2)}{\@@@Figure[#1](#2)[scale=1]}}
\def\@@@Figure[#1](#2)[#3]#4#5{\begin{figure}[#1]\centering\includegraphics[#3]{#4}\caption{#5}\end{figure}}"
  );
  DefRegister!("\\titlepgskip" => Dimension!("0pt"));
  // Metadata — preserve author values.
  DefMacro!("\\doi{}", "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!(
    "\\address[]{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#2}"
  );
  DefMacro!(
    "\\tfootnote{}",
    "\\@add@frontmatter{ltx:note}[role=titlenote]{#1}"
  );
});
