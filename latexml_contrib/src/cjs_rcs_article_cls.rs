//! cjs-rcs-article.cls (The Canadian Journal of Statistics, TeX Live 2025): the class is loaded raw, and the title-page
//! environments it makes with environ.sty's `\NewEnviron` — `{englishabstract}`, `{frenchabstract}` (:379-382
//! `\global\let\CJS@englishabstract\BODY`) and `{keywords}` (:383-386) — keep their body in stores only its
//! `\maketitle` reads (:414-480: each abstract under `\selectlanguage{<language>}\paragraph{\abstractname}`, the
//! keywords as an inline list joined by "; ", :387-393), which the kernel's locked `\maketitle` never runs: both
//! abstracts and the keywords were lost (2608.13762). The environments keep the class's environ form, their body
//! handed to the frontmatter instead of the store: each abstract an `ltx:abstract` in its language, named as the class
//! names it, the keywords with their items joined as the class's list joins them. (`{classification}`, :394-404, is
//! the same shape; no witness uses it.)
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("cjs-rcs-article", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  // (each body handed on as its tokens, `\BODY` being local to the environment, the frontmatter digested later)
  RawTeX!(
    r"\def\lx@cjs@abstract#1#2#3{\begingroup\csname captions#2\endcsname\expandafter\global\expandafter\let\csname lx@cjs@abstractname@#2\endcsname\abstractname\endgroup\lx@add@frontmatter{ltx:abstract}[name={\csname lx@cjs@abstractname@#2\endcsname},xml:lang=#3]{#1}}
\def\lx@cjs@items{\def\item{\def\item{\unskip; }}}
\def\lx@cjs@keywords#1{\lx@add@keywords[name={\iflanguage{french}{\CJS@keywordsname@fr}{\CJS@keywordsname@en}}]{{\lx@cjs@items#1}}}
\RenewEnviron{englishabstract}{\expandafter\lx@cjs@abstract\expandafter{\BODY}{english}{en}}
\RenewEnviron{frenchabstract}{\expandafter\lx@cjs@abstract\expandafter{\BODY}{french}{fr}}
\RenewEnviron{keywords}{\expandafter\lx@cjs@keywords\expandafter{\BODY}}"
  );
});
