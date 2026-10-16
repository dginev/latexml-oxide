//! breakurl.sty — breakable URLs (should be loaded after hyperref)
//! Perl: breakurl.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // breakurl.sty:35-36: its dependencies, whose `\ifpdf` (K6: `\pdfoutput`'s) decides its mode below.
  RequirePackage!("xkeyval");
  RequirePackage!("ifpdf");
  // breakurl.sty:44 reads its options with `\ProcessOptionsX`, which marks them processed: `\@curroptions` stays as it was.
  key_options_processed()?;
  // Should be loaded after hyperref.
  Let!("\\burl", "\\url");
  // Note that the arguments seem backwards in the documentation!
  // (at least, the way pdflatex processes it)
  DefMacro!("\\burlalt Semiverbatim Semiverbatim", "\\href{#2}{#1}");
  Let!("\\urlalt", "\\burlalt");
  // breakurl.sty:71-104, :186-187: the lists of characters a URL may break before and after, and the `\ifx` chain
  // built from them (`\burl@defifstructure`, run at the end of the package). `\burl` is `\url` here, which reads
  // neither, but a style extending the lists calls them: dgruyter.sty:2054-2058 `\hyper@normalise
  // \burl@addtocharlistafter{=}` `\burl@defifstructure` (1503.04650, 1711.03341). The lists start as breakurl's
  // (`%`; `:/.?#&_,;!`, :77-81, there read under `\hyper@normalise`'s catcodes). Under pdfTeX's PDF output the
  // package stops before them (:38-52 `\ifpdf … \expandafter\endinput`), and dgruyter calls them only in DVI mode
  // (:2052 `\ifnum\pdfoutput=\z@`).
  RawTeX!(r"\ifpdf\else
\def\burl@addtocharlistbefore{\g@addto@macro\burl@charlistbefore}
\def\burl@addtocharlistafter{\g@addto@macro\burl@charlistafter}
\xdef\burl@charlistbefore{\@percentchar}
\xdef\burl@charlistafter{:/.?\string#\string&\string_,;!}
\def\burl@growmif#1#2{%
  \g@addto@macro\burl@mif{\def\burl@ttt{#1}\ifx\burl@ttt\@nextchar#2\else}%
}
\def\burl@growmfi{%
  \g@addto@macro\burl@mfi{\fi}%
}
\def\burl@defifstructure{%
  \let\burl@mif\empty
  \let\burl@mfi\empty
  \expandafter\@tfor\expandafter\@nextchar\expandafter:\expandafter=%
    \burl@charlistbefore\do{%
    \expandafter\burl@growmif\@nextchar\@burl@breakbeforetrue
    \burl@growmfi
  }%
  \expandafter\@tfor\expandafter\@nextchar\expandafter:\expandafter=%
    \burl@charlistafter\do{%
    \expandafter\burl@growmif\@nextchar\@burl@breakaftertrue
    \burl@growmfi
  }%
}
\newif\if@burl@breakbefore
\newif\if@burl@breakafter
\burl@defifstructure
\fi");
});
