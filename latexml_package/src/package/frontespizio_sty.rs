//! frontespizio.sty — the Italian thesis title page.
//!
//! **Why a binding exists for this package** (the perfect-kernel mission
//! writes no new bindings except where raw interpretation is structurally
//! impossible): frontespizio's default `write` mode does not typeset the
//! title page at all — its content macros are redirected to a separate
//! `\jobname-frn.tex` (frontespizio.sty:207-229, `\front@write`), which the
//! user compiles with a SECOND pdflatex run and re-includes as a graphic
//! (:606-616 `\includegraphics{\jobname-frn}`). That external compilation is
//! out of this engine's scope, exactly like shell-escape; raw-loaded, the
//! `frontespizio` environment yields an empty `<titlepage>` and every one of
//! the six shipped manuals loses its whole content (S3 recall 0/27; Perl,
//! with no binding, loses it identically — a user-approved surpass shape).
//!
//! The package carries its own inline route: with `[nowrite,infront]` the
//! content macros STORE their values (:260-309) and
//! `\preparefrontpagestandard` / `\preparefrontpagesuftesi` (:311-538)
//! typeset the page inside `\titlepage…\endtitlepage`. The binding forces
//! that route at load time (the `infront` token registers, :158-171, are
//! only allocated then), runs the font selection the package would have
//! written to the external file (:151-156), and makes the environment's end
//! call the shape's prepare macro. xcolor is loaded for the `suftesi`
//! shape's `\color`/`Maroon` (:118-131 writes it to the external file).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("xcolor", options => vec!["svgnames".to_string()]);
  // The document's own options (`suftesi`, `sans`, …) plus the inline
  // regime, handed to the raw load's `\ProcessOptions`.
  let mut opts: Vec<String> = lookup_vecdeque("opt@frontespizio.sty")
    .map(|v| v.iter().map(|o| o.to_string()).collect())
    .unwrap_or_default();
  opts.retain(|o| o != "write" && o != "onlyinclude");
  opts.push("nowrite".to_string());
  opts.push("infront".to_string());
  InputDefinitions!("frontespizio", noltxml => true, extension => Some(Cow::Borrowed("sty")),
    handleoptions => true, options => opts);
  // The environment, inline: begin selects the fonts (`\front@thefont` =
  // `\fontoptionnormal`/`\fontoptionsans`, :151-156, otherwise written to
  // the external file), end typesets the page through the shape's prepare
  // macro (:311/:421), which opens and closes `titlepage` itself.
  // `\Preambolo{…}` / `Preambolo*`: preamble material for the external
  // title-page document, defined only in the write branch (:186-187, written
  // out) and the empty branch (:541-542, gobbled) — not in the store branch
  // the binding forces. Inline they are EXECUTED, so a `\newcommand` made
  // there (examplec's `\compring`, used in `\Titolo`) exists, while the
  // package loaders they carry (`\usepackage{fourier}`, `\setmainfont`)
  // are gobbled: the external document's packages are not this document's.
  // The environment form collects its body (environ's `\Collect@Body`, as
  // the package itself does) and runs it AFTER the environment's group
  // closes (`\aftergroup`), so the definitions are not local to it.
  RawTeX!(r"\makeatletter
\def\frontespizio{\front@thefont}
\def\endfrontespizio{\csname preparefrontpage\front@shape\endcsname}
\def\lx@front@ignorepkg{\@ifnextchar[{\lx@front@ignorepkg@}{\lx@front@ignorepkg@[]}}
\def\lx@front@ignorepkg@[#1]#2{}
\def\lx@front@preamble@begin{%
  \let\lx@front@saved@usepackage\usepackage
  \let\lx@front@saved@RequirePackage\RequirePackage
  \let\lx@front@saved@setmainfont\setmainfont
  \let\lx@front@saved@setsansfont\setsansfont
  \let\usepackage\lx@front@ignorepkg
  \let\RequirePackage\lx@front@ignorepkg
  \let\setmainfont\lx@front@ignorepkg
  \let\setsansfont\lx@front@ignorepkg}
\def\lx@front@preamble@end{%
  \let\usepackage\lx@front@saved@usepackage
  \let\RequirePackage\lx@front@saved@RequirePackage
  \let\setmainfont\lx@front@saved@setmainfont
  \let\setsansfont\lx@front@saved@setsansfont}
\renewcommand{\Preambolo}[1]{\lx@front@preamble@begin#1\lx@front@preamble@end}
\newtoks\lx@front@preamble@toks
\def\lx@front@preamble@store#1{\global\lx@front@preamble@toks{#1}}
\def\lx@front@preamble@run{\lx@front@preamble@begin\the\lx@front@preamble@toks\lx@front@preamble@end}
\renewenvironment{Preambolo*}{\Collect@Body\lx@front@preamble@store}{\aftergroup\lx@front@preamble@run}
\makeatother");
});
