//! Stub for elife.cls (eLife journal class).
//!
//! elife.cls extends extarticle with fontspec/xetex/luatex font setup
//! and defines eLife-branded colors. The raw cls's preamble does
//! `\RequirePackage{...}` for stix/opensans/XITSMath which our system
//! handles as "missing fonts" warnings, then proceeds past the
//! `\definecolor{eLifeMediumGrey}` block — but a downstream macro
//! (\NAT@nmfmt redefinition with `\color{eLifeMediumGrey}`) somehow
//! mis-handles the color name in some contexts, surfacing as 69+
//! `unexpected:eLifeMediumGrey` errors.
//!
//! Pre-define the 4 eLife color names so they're guaranteed available.
//! Witness 2307.12956. `\figsupp` prints its supplements at the document's end, as elife.cls does (2311.00616).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("extarticle");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("graphicx");
  // Pre-load xcolor with [dvipsnames, table] so user xcolor calls
  // don't silently option-clash.
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);
  RequirePackage!("booktabs");
  RequirePackage!("hyperref");
  RequirePackage!("authblk");
  RequirePackage!("microtype");
  // elife.cls L106: \RequirePackage{natbib}
  RequirePackage!("natbib");

  // Eagerly define eLife-branded colors so user `\color{eLifeMediumGrey}`
  // calls work regardless of where the raw cls bails.
  RawTeX!("\\definecolor{eLifeDarkBlue}{HTML}{273B81}");
  RawTeX!("\\definecolor{eLifeLightBlue}{HTML}{0A9DD9}");
  RawTeX!("\\definecolor{eLifeMediumGrey}{HTML}{6D6E70}");
  RawTeX!("\\definecolor{eLifeLightGrey}{HTML}{929497}");

  // Conditional flags used by raw cls preamble.
  DefConditional!("\\if@onehalfspacing");
  DefConditional!("\\if@doublespacing");

  // elife.cls L202: \corr{email}{id} — corresponding-author marker.
  // Real def appends to internal `\@correspondence`; we just preserve
  // the email and optional id as a frontmatter note. Witness 2307.12956.
  DefMacro!(
    "\\corr{}{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{\\url{#1} (#2)}"
  );
  // \contrib[id]{name} — contributing-author note. Same approach.
  DefMacro!(
    "\\contrib[]{}",
    "\\@add@frontmatter{ltx:note}[role=contribution]{#1: #2}"
  );
  // elife.cls:392-416 `\figsupp[short]{caption}{graphic}`: in the figure, the heading "Figure N—figure supplement M."
  // with the short title (the caption when there is none; nothing for `none`); the graphic and the caption go to the
  // `suppinfo` stream (newfile.sty), which elife.cls:456-461's `\AtEndDocument` inputs after the body. The paper's
  // copy of elife.cls defers that hook (`state::copy_provides_defers`), so the binding owns `\figsupp` and prints the
  // supplements at the document's end itself, each under the figure and supplement numbers the stream's
  // `\refstepcounter`s give it (:368-377, :393-398): 2311.00616's ~1,000 words of supplement captions.
  NewCounter!("figsupp");
  NewCounter!("fsdata");
  NewCounter!("fssrccode");
  RawTeX!(
    r"\def\lx@elife@none{none}
\def\lx@elife@supphead#1#2{\def\lx@elife@arg{#1}%
  \ifx\lx@elife@arg\lx@elife@none\else
    \small\textbf{\figurename~\thefigure---figure supplement \arabic{figsupp}.} %
    \def\figsuppdata##1{}\def\figsuppsrccode##1{}%
    \ifx\lx@elife@arg\@empty\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi{#2}{#1}%
  \fi}
\def\lx@elife@suppinfo{}
\def\lx@elife@addsupp#1#2#3#4{\g@addto@macro\lx@elife@suppinfo{\lx@elife@supp{#1}{#2}{#3}{#4}}}
\def\lx@elife@supp#1#2#3#4{\setcounter{figure}{#1}\setcounter{figsupp}{#2}%
  \setcounter{fsdata}{0}\setcounter{fssrccode}{0}%
  \par\noindent\begin{minipage}{\linewidth}#4\par
  \textbf{\figurename~\thefigure---figure supplement \arabic{figsupp}.} #3\par
  \end{minipage}\par}
\AtEndDocument{\lx@elife@suppinfo}"
  );
  DefMacro!(
    "\\figsupp[]{}{}",
    r"\refstepcounter{figsupp}{\lx@elife@supphead{#1}{#2}}\par
\edef\lx@elife@nums{{\the\c@figure}{\the\c@figsupp}}\expandafter\lx@elife@addsupp\lx@elife@nums{#2}{#3}"
  );
});
