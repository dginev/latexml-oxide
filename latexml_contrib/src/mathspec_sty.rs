//! mathspec.sty — XeLaTeX math font selection over fontspec (no Perl binding).
//!
//! mathspec.sty:9-10 requires XeTeX and :60 passes its options to fontspec; its
//! setters choose the fonts of math letters and digits, which LaTeXML does not
//! model (it keeps its own math fonts). The raw file checks `\XeTeXversion`
//! (:14) and raises its own errors under the Unicode-native default persona
//! (arXiv 2605.01803 ×63, 2605.02644), so the setters are bound as the
//! argument-reading no-ops they are here. Guard:
//! `perfect_kernel_batch56::xetex_only_packages_load_under_the_default_persona`.
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("fontspec");
  // mathspec.sty:223-233 `\setmathsfont(<sets>)[<features>]{<font>}` (the set
  // list is optional: `\setmathsfont[<features>]{<font>}` is the `Special`
  // branch), `\setmathfont` its alias; :916-931 `\setallmainfonts(<sets>)…`,
  // `\setprimaryfont`, `\setallsansfonts`/`\setallmonofonts`; :150-156
  // `\setsansfonts`/`\setmonofonts`; :694 `\setminwhitespace[<n>]`.
  RawTeX!(
    r"\def\lx@mathspec@sets{\@ifnextchar(\lx@mathspec@setlist{\lx@mathspec@setlist()}}
\def\lx@mathspec@setlist(#1){\@ifnextchar[\lx@mathspec@font{\lx@mathspec@font[]}}
\def\lx@mathspec@font[#1]#2{}
\def\setmathsfont{\lx@mathspec@sets}
\let\setmathfont\setmathsfont
\def\setallmainfonts{\lx@mathspec@sets}
\def\setprimaryfont{\lx@mathspec@sets}
\def\setallsansfonts{\@ifnextchar[\lx@mathspec@font{\lx@mathspec@font[]}}
\let\setallmonofonts\setallsansfonts
\let\setsansfonts\setallsansfonts
\let\setmonofonts\setallsansfonts
\providecommand\setmathcal[2][]{}
\providecommand\setmathbb[2][]{}
\providecommand\setmathfrak[2][]{}
\newcommand\setminwhitespace[1][500]{}"
  );
});
