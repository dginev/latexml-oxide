//! No-op stub for constants.sty.
//!
//! constants.sty (TL — "Labeling and numbering constants") provides
//! `\newconstantfamily`, `\C`, `\Cl`, `\refconstant`, `\resetconstant`.
//! Its end-of-document bookkeeping (constants.sty L143-148) does a *raw*
//! `\if@filesw … \makeatletter \input\jobname.aux \fi` to re-read the
//! label `.aux` for the "constants may have changed, rerun" check.
//!
//! Our engine never opens `\@mainaux` at *runtime* (the kernel's
//! `\openout\@mainaux\jobname.aux` is baked into the dump at build
//! time, with the build-time jobname), so no in-memory `<jobname>.aux`
//! buffer exists; the raw `\input\jobname.aux` then fails with
//! `Error:missing_file:<jobname>.aux`. (Unlike LaTeX's silent `\@input`,
//! constants.sty uses the hard `\input`.)
//!
//! Perl LaTeXML never observes this: its default TEXINPUTS excludes
//! `/usr/share/texlive`, so constants.sty is reported missing-file and
//! skipped — providing ZERO constants functionality. Verified on
//! arXiv:2002.05335: Perl emits `Warning:missing_file:constants` and
//! completes (its residual errors are unrelated math-mode issues).
//!
//! The package is ported whole except that end-of-document re-read: `\Cl{<label>}` records its constant at once
//! (where constants.sty writes `\newlabelconstant` to the `.aux` for the next run), so `\Cr{<label>}` after it gives
//! the constant and a forward one the "??" and warning of LaTeX's first run. Run 340: `\Cr` was the first error of 253
//! papers under the former no-op stub, which also dropped every `\C`'s C_n (2508.09984, 2103.05634, 2508.14888).
//! The `\null` closing each `\@set…constant` branch (an empty box, ending the number or name the
//! `\expandafter` produced) is `\relax` here, which ends it the same way without an empty box in the formula.
//!
//! Witnesses (CONVERR_1 → OK): 2004.04403, 2002.05335, 1501.05396,
//! 1503.01423, 1505.03199, 1506.02544, 1506.03382, 1506.03705,
//! 1508.03002, 1509.02505, 1512.06906, 1601.01096, 1603.05947,
//! 1606.09238, … (70 total).

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("keyval");
  // constants.sty :21-170 (TL 2025), but for `\labelconstant` (:150-153, an `.aux` write) and the `\AtEndDocument`
  // re-read (:144-163).
  RawTeX!(r"\def\@if@constant@exists#1#2#3{\@ifundefined{cst@family@#1}{#3}{#2}}
\global\@namedef{cst@family@normal}{}
\newcounter{cst@counter@normal}
\def\cst@format@normal#1{\arabic{#1}}
\def\cst@symbol@normal{C}
\newcommand{\newconstantfamily}[2]{\@if@constant@exists{#1}{\PackageError{constants}{The family of constants '#1'
 already exists}{Use \protect\renewconstantfamily\space to override}}{%
\expandafter\def\csname cst@family@#1\endcsname{}%
\expandafter\def\csname cst@format@#1\endcsname{\cst@format@normal}%
\expandafter\def\csname cst@symbol@#1\endcsname{\cst@symbol@normal}%
\expandafter\newcounter{cst@counter@#1}%
\define@key{constants}{format}{\expandafter\def\csname cst@format@#1\endcsname{##1}}%
\define@key{constants}{symbol}{\expandafter\def\csname cst@symbol@#1\endcsname{##1}}%
\define@key{constants}{reset}{\@addtoreset{cst@counter@#1}{##1}}%
\setkeys{constants}{#2}}}
\newcommand{\renewconstantfamily}[2]{\@if@constant@exists{#1}{%
\define@key{constants}{format}{\expandafter\def\csname cst@format@#1\endcsname{##1}}%
\define@key{constants}{symbol}{\expandafter\def\csname cst@symbol@#1\endcsname{##1}}%
\define@key{constants}{reset}{\@addtoreset{cst@counter@#1}{##1}}%
\setkeys{constants}{#2}}{\PackageError{constants}{The family of constants '#1' already exists}{Use
 \protect\renewconstantfamily\space to override}}}
\def\G@refundefinedconstanttrue{\gdef\@refundefinedconstant{\@latex@warning@no@line{There were undefined references
 to constants}}}
\let\@refundefinedconstant\relax
\def\cst@tmp@format{cst@undefined@format}
\def\cst@tmp@symbol{cst@undefined@symbol}
\long\def\@firstofthree#1#2#3{#1}
\long\def\@secondofthree#1#2#3{#2}
\long\def\@thirdofthree#1#2#3{#3}
\def\cst@undefined@format#1{}
\def\cst@undefined@symbol{\nfss@text{\reset@font\textrm{C\textsuperscript{st}\textbf{??}}}}
\def\@setrefconstant#1#2#3{\ifx#1\relax\protect\G@refundefinedconstanttrue\nfss@text{\reset@font\bfseries ??}%
\@latex@warning{Reference to constant `#3' on page \thepage \space undefined}\else\expandafter#2#1\relax\fi}
\def\@setfamconstant#1#2#3{\ifx#1\relax\def\cst@tmp@format{cst@undefined@format}\relax
\def\cst@tmp@symbol{cst@undefined@symbol}\relax\else\def\cst@tmp@format{cst@format@\expandafter#2#1}\relax
\def\cst@tmp@symbol{cst@symbol@\expandafter#2#1}\relax\fi}
\newcounter{cst@tmp@counter}
\def\@setcounterconstant#1#2#3{\ifx#1\relax\protect\G@refundefinedconstanttrue
\@latex@warning{Reference to constant `#3' on page \thepage \space undefined}\else
\setcounter{cst@tmp@counter}{\expandafter#2#1}\relax\fi}
\def\refconstant#1{\expandafter\@setrefconstant\csname cstr@#1\endcsname\@firstofthree{#1}}
\def\familyconstant#1{\expandafter\@setfamconstant\csname cstr@#1\endcsname\@thirdofthree{#1}}
\def\counterconstant#1{\expandafter\@setcounterconstant\csname cstr@#1\endcsname\@firstofthree{#1}}
\newcommand{\Cr}[1]{\counterconstant{#1}\familyconstant{#1}%
{\@nameuse{\cst@tmp@symbol}}_{\@nameuse{\cst@tmp@format}{cst@tmp@counter}}}
\newcommand{\C}[1][normal]{\@if@constant@exists{#1}{\expandafter\refstepcounterconstant{cst@counter@#1}%
{\@nameuse{cst@symbol@#1}}_{\@nameuse{cst@format@#1}{cst@counter@#1}}}{\PackageError{constants}{Family for constants
 '#1' not defined}{Check the name or use \protect\newconstantfamily}}}
\newcommand{\Cl}[2][normal]{\C[#1]\labelconstant{#2}{\string #1}}
\def\pagerefconstant#1{\expandafter\@setrefconstant\csname cstr@#1\endcsname\@secondofthree{#1}}
\def\refstepcounterconstant#1{\stepcounter{#1}\protected@edef\@currentlabelconstant{\csname p@#1\endcsname
\csname the#1\endcsname}}
\def\@currentlabelconstant{}
\def\labelconstant#1#2{\@bsphack\protected@edef\@tempa{{\@currentlabelconstant}{\thepage}{#2}}%
\expandafter\global\expandafter\let\csname cstr@#1\endcsname\@tempa\@esphack}
\newcommand{\resetconstant}[1][normal]{\@if@constant@exists{#1}{\setcounter{cst@counter@#1}{0}}{\PackageError
{constants}{Family for constants '#1' not defined}{Check the name or use \protect\newconstantfamily}}}");
});
