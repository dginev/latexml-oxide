use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: SIunits.sty.ltxml
  // Apparently siunitx is a revision and extension of SIunits
  // Not quite backwards compatible, but worth a try...
  RequirePackage!("siunitx");

  // Apparently similar, but expects the numbers to be already formatted?
  // (things like \times, ^, etc appear)
  Let!("\\unit", "\\SI");

  RawTeX!("\\sisetup{parse-numbers = false, input-product = \\times,}");

  DefMacro!("\\squaren{}", "{#1}^{2}");

  // SIunits.sty's own unit-building commands (SIunits.sty L51-193, 266),
  // which papers call outside `\SI`/`\unit` (`\addunit{\Pa}{Pa}`,
  // `\metre\usk\reciprocal\second`, `\degreecelsius`; witnesses 0709.1649,
  // 2103.01586, 1904.06264, 2407.13230). `\providecommand` keeps whatever
  // siunitx owns: its unit names are
  // `\relax` outside `\si`, which `\providecommand` treats as undefined, and `\si`
  // rebinds them to its own unit objects inside its argument.
  RawTeX!(r"\makeatletter
\providecommand\SI@fstyle[1]{\mathrm{#1}}
\providecommand{\addunit}[2]{\newcommand{#1}{\ensuremath{\SI@fstyle{#2}}}}
\providecommand{\addprefix}[2]{\newcommand{#1}{\ensuremath{\SI@fstyle{#2}}}}
\providecommand*{\usk}{\ensuremath{\;}}
\providecommand*{\unitskip}[1]{\renewcommand{\usk}{\ensuremath{#1}}}
\providecommand*{\per}{\ensuremath{\SI@fstyle{/}}}
\providecommand{\power}[2]{\ensuremath{\SI@fstyle{#1}^{\SI@fstyle{#2}}}}
\providecommand*{\squared}{\ensuremath{^{\SI@fstyle{2}}}}
\providecommand*{\cubic}[1]{\power{#1}{3}}
\providecommand*{\cubed}{\ensuremath{^{\SI@fstyle{3}}}}
\providecommand*{\fourth}[1]{\power{#1}{4}}
\providecommand*{\SIminus}{\ensuremath{-}}
\providecommand*{\reciprocal}[1]{\power{#1}{\SIminus1}}
\providecommand*{\rp}[1]{\power{#1}{\SIminus1}}
\providecommand*{\rpsquare}[1]{\power{#1}{\SIminus2}}
\providecommand*{\rpsquared}{\ensuremath{^{\SI@fstyle{\SIminus2}}}}
\providecommand*{\rpcubic}[1]{\power{#1}{\SIminus3}}
\providecommand*{\rpcubed}{\ensuremath{^{\SI@fstyle{\SIminus3}}}}
\providecommand*{\rpfourth}[1]{\power{#1}{\SIminus4}}
\providecommand{\degreecelsius}{\ensuremath{\lx@siunits@degreecelsius}}
\providecommand*\SIunits[1][thickspace,thickqspace]{\@for\SIunits@@:=#1\do{\SIunits@execopt\SIunits@@}}
\providecommand*\SIunits@execopt[1]{\@ifundefined{SIunits@opt@#1}%
  {\PackageWarning{SIunits}{Requested option `#1' not provided}}%
  {\@nameuse{SIunits@opt@#1}}}
\providecommand*\SIunits@opt@cdot{\renewcommand*{\usk}{\ensuremath{\cdot}}}
\providecommand*\SIunits@opt@thickspace{\renewcommand*{\usk}{\ensuremath{\;}}}
\providecommand*\SIunits@opt@mediumspace{\renewcommand*{\usk}{\ensuremath{\:}}}
\providecommand*\SIunits@opt@thinspace{\renewcommand*{\usk}{\ensuremath{\,}}}
\providecommand*\SIunits@opt@thickqspace{}
\providecommand*\SIunits@opt@mediumqspace{}
\providecommand*\SIunits@opt@thinqspace{}
\makeatother");
  // `\degreecelsius` (L266, `\degree\Celsius`: a raised ring before an upright C) as the one glyph ℃, as gensymb's
  // `\celsius`, so a number before it parses as the quantity it is.
  DefMath!("\\lx@siunits@degreecelsius", None, "\u{2103}", alias => "\\degreecelsius");
  // `\DeclareMathSymbol{\upmu}{\mathord}{greek}{"16}` (L51), as upgreek's.
  DefMath!("\\upmu", '\u{03BC}', font => { shape => "upright", forceshape => true });

  // NOTE: do NOT `six_enable_unit_macros(true)` here. The real SIunits package
  // defines short single-letter unit macros (`\m`=metre, `\s`=second, `\g`,
  // `\A`, …), but Perl's `SIunits.sty.ltxml` is a thin shim — it only
  // `RequirePackage('siunitx')` + `\squaren`, defining NONE of those macros
  // (siunitx uses `\si{\metre}` syntax instead). Enabling them here was a
  // Rust-only divergence: defining `\m` clobbers the extremely common user
  // macro `\newcommand{\m}[1]{\(#1\)}` (a LaTeX `\newcommand` of an
  // already-defined CS is silently ignored), so every `\m{…}` rendered as the
  // metre symbol in TEXT mode → 753× `^`/`_` "can only appear in math mode"
  // (witness 1509.04521, `\usepackage[squaren]{SIunits}` + `\newcommand{\m}`).
  // Matching Perl (which leaves `\m` for the document) fixes it; SIunits docs
  // that genuinely use `\m`/`\metre` standalone already fail in Perl too.
});
