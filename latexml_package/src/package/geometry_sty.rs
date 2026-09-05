//! geometry.sty — page layout.
//!
//! Perl LaTeXML (`geometry.sty.ltxml`) makes every geometry macro a no-op:
//! page geometry is meaningless for the reflowable HTML body, so `\textwidth`
//! stays at the class default (345pt). We keep that behaviour for the HTML flow
//! — a `\rule{0.5\linewidth}`, a text minipage, `\includegraphics[width=
//! \linewidth]` etc. are all sized from the class default exactly as in Perl.
//!
//! **Divergence from Perl (OXIDIZED_DESIGN #99):** a *measured SVG graphic*
//! that reads `\linewidth` — a `tcolorbox`, `tikzpicture`, or bare `pgfpicture`
//! — is emitted as a fixed-size `<svg>` whose aspect ratio is baked at
//! conversion time, so ignoring the real page width there makes the picture
//! `0.5 x 345pt` instead of the `0.5 x 472pt` the PDF draws (letterpaper minus
//! the document's margins). The narrow interior over-wraps the content, roughly
//! doubling the box height and pushing text through the border
//! (arXiv:2605.29955, Figure 1). So we DO compute the geometry text width/height
//! and inject it into `\linewidth`/`\hsize`/`\columnwidth`/`\textwidth` — but
//! ONLY inside SVG-producing pictures, never into the main HTML flow. The guard
//! `\ifdim\linewidth=\textwidth` applies it only at the top level, leaving a
//! locally-reduced `\linewidth` (nested minipage/parbox) alone.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Dependencies — Perl L22-25
  RequirePackage!("keyval");
  RequirePackage!("ifpdf");
  RequirePackage!("ifvtex");
  RequirePackage!("ifxetex");

  RawTeX!(r#"\makeatletter
%% ---- state ---------------------------------------------------------------
\newdimen\Gm@pw \newdimen\Gm@ph            % paper width/height
\newdimen\Gm@l \newdimen\Gm@r              % left/right margins
\newdimen\Gm@t \newdimen\Gm@b              % top/bottom margins
\newdimen\Gm@tw \newdimen\Gm@th            % computed text width/height (SVG scope)
\newdimen\Gm@doctw                         % class-default \textwidth (the full column)
\newdimen\Gm@bindingoffset                 % geometry.sty L51; read by classes
\newdimen\Gm@lmargin \newdimen\Gm@rmargin \newdimen\Gm@tmargin \newdimen\Gm@bmargin
\newdimen\Gm@width \newdimen\Gm@height    % geometry.sty:767 names read by classes (stocksize-doc)
                                           % (tudapub.cls L506 \g_ptxcd_headwidth_dim)
\newif\ifGm@lset \newif\ifGm@rset \newif\ifGm@tset \newif\ifGm@bset
\newif\ifGm@twset \newif\ifGm@thset
\Gm@pw=\paperwidth \Gm@ph=\paperheight
\Gm@tw=\textwidth  \Gm@th=\textheight
\Gm@doctw=\textwidth

%% ---- helpers -------------------------------------------------------------
% Split a value that is either scalar `v` or a pair `{a,b}` into \Gm@vA/\Gm@vB.
% For a scalar, both halves are `v` (so margin=1in => all four margins 1in).
\def\Gm@pair#1{\Gm@pair@#1,#1,\@nil}
\def\Gm@pair@#1,#2,#3\@nil{\def\Gm@vA{#1}\def\Gm@vB{#2}}

%% ---- keys (family Gm, prefix KV) -----------------------------------------
% Paper sizes
\define@key{Gm}{paperwidth}{\Gm@pw=#1\relax}
\define@key{Gm}{paperheight}{\Gm@ph=#1\relax}
\define@key{Gm}{papersize}{\Gm@pair{#1}\Gm@pw=\Gm@vA\relax\Gm@ph=\Gm@vB\relax}
\define@key{Gm}{letterpaper}[]{\Gm@pw=8.5in\Gm@ph=11in }
\define@key{Gm}{legalpaper}[]{\Gm@pw=8.5in\Gm@ph=14in }
\define@key{Gm}{executivepaper}[]{\Gm@pw=7.25in\Gm@ph=10.5in }
\define@key{Gm}{a4paper}[]{\Gm@pw=210mm\Gm@ph=297mm }
\define@key{Gm}{a5paper}[]{\Gm@pw=148mm\Gm@ph=210mm }
\define@key{Gm}{b5paper}[]{\Gm@pw=176mm\Gm@ph=250mm }
\define@key{Gm}{landscape}[]{\Gm@pw=\paperheight\Gm@ph=\paperwidth}
% Margins (each with the aliases geometry accepts)
\define@key{Gm}{left}{\Gm@l=#1\relax\Gm@lsettrue}
\define@key{Gm}{lmargin}{\Gm@l=#1\relax\Gm@lsettrue}
\define@key{Gm}{inner}{\Gm@l=#1\relax\Gm@lsettrue}
\define@key{Gm}{right}{\Gm@r=#1\relax\Gm@rsettrue}
\define@key{Gm}{rmargin}{\Gm@r=#1\relax\Gm@rsettrue}
\define@key{Gm}{outer}{\Gm@r=#1\relax\Gm@rsettrue}
\define@key{Gm}{top}{\Gm@t=#1\relax\Gm@tsettrue}
\define@key{Gm}{tmargin}{\Gm@t=#1\relax\Gm@tsettrue}
\define@key{Gm}{bottom}{\Gm@b=#1\relax\Gm@bsettrue}
\define@key{Gm}{bindingoffset}{\Gm@bindingoffset=#1\relax}
\define@key{Gm}{bmargin}{\Gm@b=#1\relax\Gm@bsettrue}
% margin={h,v}: h -> left+right, v -> top+bottom (scalar -> all four)
\define@key{Gm}{margin}{\Gm@pair{#1}%
  \Gm@l=\Gm@vA\relax\Gm@lsettrue \Gm@r=\Gm@vA\relax\Gm@rsettrue
  \Gm@t=\Gm@vB\relax\Gm@tsettrue \Gm@b=\Gm@vB\relax\Gm@bsettrue}
% hmargin={l,r}
\define@key{Gm}{hmargin}{\Gm@pair{#1}%
  \Gm@l=\Gm@vA\relax\Gm@lsettrue \Gm@r=\Gm@vB\relax\Gm@rsettrue}
% vmargin={t,b}
\define@key{Gm}{vmargin}{\Gm@pair{#1}%
  \Gm@t=\Gm@vA\relax\Gm@tsettrue \Gm@b=\Gm@vB\relax\Gm@bsettrue}
% Body dimensions given directly
\define@key{Gm}{textwidth}{\Gm@tw=#1\relax\Gm@twsettrue}
\define@key{Gm}{width}{\Gm@tw=#1\relax\Gm@twsettrue}
\define@key{Gm}{totalwidth}{\Gm@tw=#1\relax\Gm@twsettrue}
\define@key{Gm}{textheight}{\Gm@th=#1\relax\Gm@thsettrue}
\define@key{Gm}{height}{\Gm@th=#1\relax\Gm@thsettrue}
\define@key{Gm}{totalheight}{\Gm@th=#1\relax\Gm@thsettrue}
% scale=s (or {sh,sv}) — fraction of the paper size
\define@key{Gm}{scale}{\Gm@pair{#1}%
  \Gm@tw=\Gm@vA\Gm@pw\Gm@twsettrue \Gm@th=\Gm@vB\Gm@ph\Gm@thsettrue}

%% ---- recompute the text box from the collected keys ----------------------
\def\Gm@recalc{%
  \ifGm@twset\else
    \ifGm@lset\ifGm@rset
      \Gm@tw=\Gm@pw \advance\Gm@tw-\Gm@l \advance\Gm@tw-\Gm@r
      \advance\Gm@tw-\Gm@bindingoffset
    \fi\fi
  \fi
  \ifGm@thset\else
    \ifGm@tset\ifGm@bset
      \Gm@th=\Gm@ph \advance\Gm@th-\Gm@t \advance\Gm@th-\Gm@b
    \fi\fi
  \fi}

% \geometry{keys} — parse (ignoring unimplemented keys) and recompute.
\def\geometry#1{\setkeys*{Gm}{#1}\Gm@recalc}
\def\newgeometry#1{\setkeys*{Gm}{#1}\Gm@recalc}
\let\restoregeometry\@empty
\def\savegeometry#1{}
\def\loadgeometry#1{}

% Package options — `\usepackage[left=...,margin=...]{geometry}` — are routed
% through the same key parser; each comma item arrives as \CurrentOption.
\def\Gm@opt#1{\setkeys*{Gm}{#1}}
\DeclareOption*{\expandafter\Gm@opt\expandafter{\CurrentOption}}
\ProcessOptions*\relax
\Gm@recalc

%% ---- SVG-scope injection -------------------------------------------------
% Applied at the start of an SVG-producing picture: raise \linewidth (and the
% siblings measured graphics read) to the geometry text width, but ONLY when the
% picture is at the FULL column width — compared against the class-default
% \textwidth captured at load (\Gm@doctw), NOT the live \textwidth. A minipage
% sets \textwidth=\linewidth locally, so `\linewidth=\textwidth` is true inside
% one too; comparing to \Gm@doctw instead keeps a reduced \linewidth (nesting
% minipage/parbox) untouched. Scoped to the picture's group, so the surrounding
% HTML flow is never touched.
\def\Gm@applysvgwidth{%
  \ifdim\linewidth=\Gm@doctw
    \linewidth=\Gm@tw \columnwidth=\Gm@tw \hsize=\Gm@tw \textwidth=\Gm@tw
    \textheight=\Gm@th
  \fi}

% Install the picture hooks after all packages have loaded (tcolorbox/tikz/pgf
% are raw-loaded and only then defined). Prepend, so the width is in force
% before the box reads its `width=...\linewidth` key.
\AtBeginDocument{%
  \@ifundefined{tcolorbox}{}{%
    \let\Gm@orig@tcolorbox\tcolorbox
    \def\tcolorbox{\Gm@applysvgwidth\Gm@orig@tcolorbox}}%
  \@ifundefined{tikzpicture}{}{%
    \let\Gm@orig@tikzpicture\tikzpicture
    \def\tikzpicture{\Gm@applysvgwidth\Gm@orig@tikzpicture}}%
  \@ifundefined{pgfpicture}{}{%
    \let\Gm@orig@pgfpicture\pgfpicture
    \let\Gm@orig@endpgfpicture\endpgfpicture
    % \pgfpicture opens its own group internally, so wrap it in an explicit
    % group of our own to keep the width change from leaking past the picture.
    \def\pgfpicture{\begingroup\Gm@applysvgwidth\Gm@orig@pgfpicture}%
    \def\endpgfpicture{\Gm@orig@endpgfpicture\endgroup}}%
}
\makeatother"#);
});
