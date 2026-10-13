//! Stub for jfm.cls (Journal of Fluid Mechanics).
//!
//! Author-bundled class commonly used by JFM submissions. Defines a few
//! frontmatter helpers that aren't in standard article (`\aff`, `\corresp`,
//! `\affiliation`). The cls itself isn't raw-loaded reliably; stub the
//! author-facing macros so submissions don't error on undefined CS.
//! Witness 2312.07468.
use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("graphicx");
  RequirePackage!("hyperref");
  RequirePackage!("natbib");
  // jfm.cls:663-712 builds subeqnarray in (2512.18771: `{subeqnarray}` undefined, its body read as text).
  RequirePackage!("subeqnarray");
  // jfm.cls (v1.27/v1.28 headers, which every copy keeps through the press's 2020 version) :58, :72
  // `\newif\ifCUPmtlplainloaded`, set by the `prodtf` option for the press's own fonts (CUPTimes, jfm2esym, :79; not
  // distributed): the class and the template preamble test it around their font checks (`\ifCUPmtlplainloaded \else
  // \checkfont{eurm10} …`; run 340: 597 papers' first error; witnesses 2103.10260, 2103.05851, 1904.02498). The 2020
  // version's `lineno` option (:172, :186, :2239) loads the lineno-FLM.sty the papers ship at the start of the
  // document (`\nolinenumbers` in a title: 2401.13479, 2406.10975).
  RawTeX!(r"\newif\ifCUPmtlplainloaded\newif\iflineno");
  DeclareOption!("prodtf", "\\CUPmtlplainloadedtrue");
  DeclareOption!("lineno", "\\linenotrue");
  DeclareOption!(None, {});
  ProcessOptions!();
  RawTeX!(r"\iflineno\AtBeginDocument{\usepackage{lineno-FLM}}\fi");
  // The 2020 version's caption (:1837, `labelsep=period`, the "Figure 1." the older versions' own `\@makecaption`
  // prints too, :1042-1047), whose `\captionsetup` papers call (run 340: 51; 2406.15733).
  RequirePackage!("caption", options => vec![s!("labelsep=period")]);
  // jfm.cls:1176-1183 `\checkfont` and the font checks every version runs (:1324-1365): amsbsy, and upmath when the
  // Euler fonts are found and the paper ships upmath.sty (TeX Live has none, so `\upi` is the class's `\pi`; witnesses
  // 2103.06961, 1904.11981). The msam10 check's `\let\le=\leqslant` (74 of 119 sampled copies; the 2020 version
  // comments it out) is a glyph shape, the same relation, and is not carried.
  RawTeX!(
    r"\newif\iffontfound
\newcommand\checkfont[1]{\batchmode\font\test=#1\relax\errorstopmode\fontfoundfalse\ifx\test\nullfont\else\fontfoundtrue\fi}
\ifCUPmtlplainloaded\else
  \IfFileExists{amsbsy.sty}{\usepackage{amsbsy}}{\providecommand\boldsymbol[1]{\mbox{\boldmath $##1$}}}
  \checkfont{eurm10}
  \iffontfound\IfFileExists{upmath.sty}{\usepackage{upmath}}{\providecommand\upi{\pi}}\else\providecommand\upi{\pi}\fi
\fi"
  );
  // The author conveniences of the class versions (union of 119 sampled copies): what one version defines another may
  // not, so a paper whose copy lacks one defines it itself, and its own `\newcommand` wins over ours
  // (`DefinitionOrigin::Fallback`, as a fallback class's guesses do). Run 340 first errors: `\Rey` 321, `\mathsfbi`
  // 136, `\ns` 97, `\ls` 88, `\bnabla` 82, `\p` 52, `\bcdot` 50, `\Pran` 33 (witnesses 2508.02297, 2604.04559,
  // 1309.5858, 2103.06764, 2103.16498, 2103.09196, 1904.11400, 1904.04853).
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    // :1107-1108 author-list spacing; :1269-1275 multi-letter symbols (`\Ray` in 2211.00559's copy); :1263-1266 bold
    // symbols; :1278-1291 Bessel functions, sans-serif letters, hats; :1293-1321 the template's shorthands.
    RawTeX!(
      r"\newcommand\ls{\kern.15em\relax}
\newcommand\ns{\kern.5em\relax}
\newcommand\dynpercm{\nobreak\mbox{$\;$dyn\,cm$^{-1}$}}
\newcommand\cmpermin{\nobreak\mbox{$\;$cm\,min$^{-1}$}}
\providecommand\bnabla{\boldsymbol{\nabla}}
\providecommand\bcdot{\boldsymbol{\cdot}}
\newcommand\biS{\boldsymbol{S}}
\newcommand\etb{\boldsymbol{\eta}}
\newcommand\Real{\mbox{Re}}
\newcommand\Imag{\mbox{Im}}
\newcommand\Rey{\mbox{\textit{Re}}}
\newcommand\Pran{\mbox{\textit{Pr}}}
\newcommand\Pen{\mbox{\textit{Pe}}}
\newcommand\Ray{\mbox{\textit{Ra}}}
\newcommand\Ai{\mbox{Ai}}
\newcommand\Bi{\mbox{Bi}}
\newcommand\rmJ{{\rm J}}
\newcommand\rmY{{\rm Y}}
\newcommand\rmK{{\rm K}}
\newcommand\rmI{{\rm I}}
\newcommand\ssC{\mathsf{C}}
\newcommand\sfsP{\mathsfi{P}}
\newcommand\slsQ{\mathsfbi{Q}}
\newcommand\hatp{\skew3\hat{p}}
\newcommand\hatR{\skew3\hat{R}}
\newcommand\hatRR{\skew3\hat{\hatR}}
\newcommand\doubletildesigma{\skew2\tilde{\skew2\tilde{\Sigma}}}
\newsavebox{\astrutbox}
\sbox{\astrutbox}{\rule[-5pt]{0pt}{20pt}}
\newcommand{\astrut}{\usebox{\astrutbox}}
\newcommand\GaPQ{\ensuremath{G_a(P,Q)}}
\newcommand\GsPQ{\ensuremath{G_s(P,Q)}}
\newcommand\p{\ensuremath{\partial}}
\newcommand\tti{\ensuremath{\rightarrow\infty}}
\newcommand\kgd{\ensuremath{k\gamma d}}
\newcommand\shalf{\ensuremath{{\scriptstyle\frac{1}{2}}}}
\newcommand\sh{\ensuremath{^{\shalf}}}
\newcommand\smh{\ensuremath{^{-\shalf}}}
\newcommand\squart{\ensuremath{{\textstyle\frac{1}{4}}}}
\newcommand\thalf{\ensuremath{{\textstyle\frac{1}{2}}}}
\newcommand\Gat{\ensuremath{\widetilde{G_a}}}
\newcommand\ttz{\ensuremath{\rightarrow 0}}
\newcommand\ndq{\ensuremath{\frac{\mbox{$\partial$}}{\mbox{$\partial$} n_q}}}
\newcommand\sumjm{\ensuremath{\sum_{j=1}^{M}}}
\newcommand\pvi{\ensuremath{\int_0^{\infty}\mskip\ifCUPmtlplainloaded -30mu\else -33mu\fi -\quad}}
\newcommand\etc{etc.\ }
\newcommand\eg{e.g.\ }"
    );
    // 2020 version :173-187: the sans-serif math alphabets, bold-italic sans from Helvetica where its font
    // description is installed (TeX Live has t1phv.fd), as pdflatex picks it; `\mathbi` (:1443) is `\boldsymbol`.
    RawTeX!(
      r"\DeclareTextFontCommand\textsfi{\usefont{OT1}{cmss}{m}{sl}}
\DeclareMathAlphabet\mathsfi{OT1}{cmss}{m}{sl}
\DeclareTextFontCommand\textsfb{\usefont{OT1}{cmss}{bx}{n}}
\DeclareMathAlphabet\mathsfb{OT1}{cmss}{bx}{n}
\DeclareTextFontCommand\textsfbi{\usefont{T1}{phv}{b}{it}}
\DeclareMathAlphabet\mathsfbi{T1}{phv}{b}{it}
\let\mathbi\boldsymbol"
    );
    // :385-389 a single appendix (`\appendix` titled without a letter); :1378 `\aunote` (`\corresp`'s twin); the 2020
    // version's `\corresau` (:2193), printed under the title as "Corresponding author: …" (2604.22963).
    RawTeX!(
      r"\newif\ifappendix\newif\ifoneappendix\newcommand\oneappendix{\global\oneappendixtrue\appendix}"
    );
    DefMacro!("\\aunote{}", "\\thanks{#1}");
    DefMacro!(
      "\\corresau{}",
      "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}"
    );
  }

  // jfm.cls L1498: \newcommand{\backsection}[2][\backsectionname]{...\textbf{#1.} #2...}
  // — a backmatter paragraph (Acknowledgements / Declaration of interests / Data
  // availability) with a bold title. Was undefined (Perl defines it). Render as the
  // faithful bold-title paragraph. The title is always given as the optional arg in
  // practice; default to "Acknowledgements". Witness 2309.14752.
  // jfm.cls: \newcommand\pagerange[1]{\gdef\@pagerange{#1}} — stores the page range
  // for the running head (no body output). Gobble (faithful: not rendered in body).
  // Was undefined. Witness 2406.18711.
  DefMacro!("\\pagerange{}", "");
  DefMacro!("\\backsectionname", "Acknowledgements");
  // Optional title arg (always supplied in practice, e.g. [Acknowledgements] /
  // [Declaration of interests] / [Data availability statement]); empty default.
  DefMacro!("\\backsection[]{}", "\\par\\noindent\\textbf{#1.} #2\\par");
  // jfm.cls L?: \def\aff#1{\ignorespaces\textsuperscript{#1}} — affil
  // marker as superscript.
  DefMacro!("\\aff{}", "\\textsuperscript{#1}");
  // jfm.cls: \def\corresp#1{\unskip\thanks{#1}} — corresponding author
  // text routed via \thanks.
  DefMacro!("\\corresp{}", "\\thanks{#1}");
  // \affiliation{...} — institution list, preserve as ltx:note.
  DefMacro!(
    "\\affiliation{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#1}"
  );
  // \keywords{...} — JFM has its own keywords macro.
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");
});
