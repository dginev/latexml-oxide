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
  // jfm.cls loads no amsmath (196 copies in a 1/20 arXiv sample): amsmath's commands for papers that use them unloaded,
  // its "Old form" error once the paper loads it; the paper's copy decides
  // (`amsmath_sty::require_amsmath_beyond_original`, KNOWN_PERL_ERRORS #993).
  amsmath_sty::require_amsmath_beyond_original(
    latexml_package::shipped_class::shipped_copy_loads("cls", "amsmath")?,
    &[],
  )?;
  RequirePackage!("amssymb");
  RequirePackage!("graphicx");
  RequirePackage!("hyperref");
  RequirePackage!("natbib");
  // jfm.cls:663-712 builds subeqnarray in (2512.18771: `{subeqnarray}` undefined, its body read as text).
  RequirePackage!("subeqnarray");
  // jfm.cls (v1.27/v1.28 headers, which every copy keeps; line numbers from 2103.10260's v1.28 copy unless said)
  // :58, :72 `\newif\ifCUPmtlplainloaded`, set by the `prodtf` option for the press's own fonts (CUPTimes, jfm2esym,
  // :79; not distributed): the class and the template preamble test it around their font checks
  // (`\ifCUPmtlplainloaded \else \checkfont{eurm10} …`; run 340: 597 papers' first error; witnesses 2103.10260,
  // 2103.05851, 1904.02498). The newer copies' `lineno` option (2604.22963's :172, :186, :2239) loads the
  // lineno-FLM.sty the papers ship at the start of the document (`\nolinenumbers` in a title: 2401.13479, 2406.10975);
  // an older copy has no such option (2605.03843's v1.25, whose paper ships no lineno-FLM.sty: `[lineno]` is an unused
  // option there), so the option is the copy's when the paper has one.
  // The paper's own copy decides what follows where the versions differ: that option; caption, which 35 of 119 sampled
  // copies load (`\captionsetup`, run 340: 51; 2406.15733); the msam10 check's `\let\le=\leqslant` (73 copies,
  // active); the bold-italic sans `\mathsfbi` (below). Without a readable copy, the newer versions' choices.
  let shipped = crate::shipped_class::shipped_class_text("jfm")?;
  RawTeX!(r"\newif\ifCUPmtlplainloaded\newif\iflineno");
  DeclareOption!("prodtf", "\\CUPmtlplainloadedtrue");
  if shipped
    .as_deref()
    .is_none_or(|text| crate::shipped_class::class_code_contains(text, "\\DeclareOption{lineno}"))
  {
    DeclareOption!("lineno", "\\linenotrue");
  }
  DeclareOption!(None, {});
  ProcessOptions!();
  RawTeX!(r"\iflineno\AtBeginDocument{\usepackage{lineno-FLM}}\fi");
  // The copy's caption load keeps its options (2608.01828's :1837 `[…,labelsep=period,…]{caption}`: "Figure 1."); the
  // copies without caption print "Figure 1." with the label in small caps from their own `\@makecaption` (0712.2739's
  // :953-958 `{\normalfont\scshape #1\ifrem@fullpt\else .\fi} #2`, `\removefullpoint` dropping the period once),
  // which the kernel's caption keeps out (`\@makecaption keeps LaTeXML's definition`): its label format is the
  // figure's and table's title format, as amsart's (ams_support_sty.rs `ams_class_caption_format`), its font the label
  // font (`\fnum@font@<type>`, sect09.rs) under the class caption fonts knob alone. The document's caption package —
  // or subcaption's or subfig's, which load it — replaces the class's `\@makecaption` ("Figure 1:" in the text font;
  // 1711.01065, 2205.10181, 2507.07213 subcaption, 1805.00129 subfig), the old subfigure package does not, nor
  // floatrow, which loads caption's kernel alone (floatrow.sty:39 `caption3`, whose stand-in is our caption binding:
  // `\lx@caption@kernel@only`; 2312.13002, 2111.12004): so the format is the class's only where caption is not loaded
  // by `\begin{document}`; `\removefullpoint` is the copy's either way. Before, both gave "Figure 1:"
  // (OXIDIZED_DESIGN_DIVERGENCES #784; guards `perfect_kernel_batch78a::{
  // jfm_copy_captions_end_the_label_with_a_period, jfm_copy_caption_package_keeps_its_options,
  // jfm_copy_caption_yields_to_caption_loaders, jfm_copy_caption_kept_with_subfigure,
  // jfm_copy_caption_kept_with_floatrow}`).
  if shipped
    .as_deref()
    .is_some_and(|text| crate::shipped_class::class_code_contains(text, "rem@fullpt"))
  {
    RawTeX!(r"\newif\ifrem@fullpt\def\removefullpoint{\global\rem@fullpttrue}");
  }
  match shipped.as_deref() {
    None => RequirePackage!("caption"),
    Some(text) if crate::shipped_class::class_text_loads(text, "caption") => {
      // The dependency scan declines a `\usepackage` it cannot read as the class's (a `\def` body, a comment block):
      // caption is loaded all the same.
      if !crate::shipped_class::require_shipped_packages("cls", &["caption"])? {
        RequirePackage!("caption");
      }
    },
    Some(text) => {
      if crate::shipped_class::class_code_contains(text, "\\@makecaption")
        && crate::shipped_class::class_code_contains(text, "rem@fullpt")
      {
        // (the class's `\@makecaption` for every kernel float, a newfloat type's as a figure's: `\lx@makecaption`; its
        // small-caps label under the class caption fonts knob alone, caption_sty.rs `CLASS_CAPTION_FONTS`)
        let fonts = if caption_sty::class_caption_fonts() {
          r"\def\fnum@font@float{\scshape}\def\fnum@font@figure{\scshape}\def\fnum@font@table{\scshape}%"
        } else {
          ""
        };
        caption_sty::class_makecaption(&s!(
          r"{fonts}
\long\def\lx@makecaption##1##2{{\lx@tag[][\ifrem@fullpt\space\else. \fi]{{\lx@fnum@@{{##1}}}}##2\global\rem@fullptfalse}}%
\long\def\format@title@figure##1{{\lx@makecaption{{figure}}{{##1}}}}\long\def\format@title@table##1{{\lx@makecaption{{table}}{{##1}}}}"
        ))?;
      }
    },
  }
  // jfm.cls:1176-1183 `\checkfont` and the font checks every version runs (:1324-1365): amsbsy, and upmath when the
  // Euler fonts are found and the paper ships upmath.sty (TeX Live has none, so `\upi` is the class's `\pi`; witnesses
  // 2103.06961, 1904.11981); msam10 is found, so amssymb loads and, where the copy has it, `\le`, `\leq`, `\ge`, `\geq`
  // become the slanted ⩽ ⩾ (:1355-1363).
  RawTeX!(
    r"\newif\iffontfound
\newcommand\checkfont[1]{\batchmode\font\test=#1\relax\errorstopmode\fontfoundfalse\ifx\test\nullfont\else\fontfoundtrue\fi}
\ifCUPmtlplainloaded\else
  \IfFileExists{amsbsy.sty}{\usepackage{amsbsy}}{\providecommand\boldsymbol[1]{\mbox{\boldmath $##1$}}}
  \checkfont{eurm10}
  \iffontfound\IfFileExists{upmath.sty}{\usepackage{upmath}}{\providecommand\upi{\pi}}\else\providecommand\upi{\pi}\fi
\fi"
  );
  if shipped.as_deref().is_none_or(slants_inequalities) {
    RawTeX!(
      r"\ifCUPmtlplainloaded\else\let\le=\leqslant\let\leq=\leqslant\let\ge=\geqslant\let\geq=\geqslant\fi"
    );
  }
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
    // The sans-serif math alphabets (2103.09196's copy :173-187): the newer copies' bold-italic sans comes from
    // Helvetica where its font descriptions are installed (they test t1phv.fd, then ot1phv.fd, and TeX Live has both,
    // so pdflatex ends with OT1/phv/b/it); the older copies' `\mathsfbi` is slanted medium cmss (0712.2739's :166).
    // `\mathbi` (:1443) is `\boldsymbol`.
    RawTeX!(
      r"\DeclareTextFontCommand\textsfi{\usefont{OT1}{cmss}{m}{sl}}
\DeclareMathAlphabet\mathsfi{OT1}{cmss}{m}{sl}
\DeclareTextFontCommand\textsfb{\usefont{OT1}{cmss}{bx}{n}}
\DeclareMathAlphabet\mathsfb{OT1}{cmss}{bx}{n}
\let\mathbi\boldsymbol"
    );
    if shipped
      .as_deref()
      .is_none_or(|text| text.contains("{phv}{b}{it}"))
    {
      RawTeX!(
        r"\DeclareTextFontCommand\textsfbi{\usefont{OT1}{phv}{b}{it}}\DeclareMathAlphabet\mathsfbi{OT1}{phv}{b}{it}"
      );
    } else {
      RawTeX!(
        r"\DeclareTextFontCommand\textsfbi{\usefont{OT1}{cmss}{m}{sl}}\DeclareMathAlphabet\mathsfbi{OT1}{cmss}{m}{sl}"
      );
    }
    // :385-389 `\oneappendix`, a single appendix (the class titles it "Appendix" without a letter, which this
    // `\appendix` still prints: a residual); :1378 `\aunote` (`\corresp`'s twin); the newest copies' `\corresau`
    // (2604.22963's :2193), printed under the title as "Corresponding author: …".
    RawTeX!(
      r"\newif\ifappendix\newif\ifoneappendix\newcommand\oneappendix{\global\oneappendixtrue\appendix}"
    );
    // The figure-caption legend symbols (1702.03412's :1380-1405: `\fullcirc`, `\opencirc`, `\fullsquare`, the line
    // styles `\full`, `\dashed`, `\dotted`, `\chain`, …; 73 of 85 copies, 29 papers: 1702.03412, 1806.08341,
    // 1909.12979) and the upright capital Greek (:1419-1429 `\let\rmDelta\Delta`, …; 2004.10578, 2207.01368).
    RawTeX!(
      r"\newcommand{\opensquare}{\mbox{$\square$}}\newcommand{\opentriangle}{\mbox{$\vartriangle$}}
\newcommand{\opentriangledown}{\mbox{$\triangledown$}}\newcommand{\opendiamond}{\mbox{$\lozenge$}}
\newcommand{\fullsquare}{\mbox{$\blacksquare$}}\newcommand{\fulldiamond}{\mbox{$\blacklozenge$}}
\newcommand{\fullstar}{\mbox{$\bigstar$}}\newcommand{\fulltriangle}{\mbox{$\blacktriangle$}}\let\fulltri=\fulltriangle
\newcommand{\fulltriangledown}{\mbox{$\blacktriangledown$}}\let\fulltridown=\fulltriangledown
\newcommand{\opencirc}{\mbox{\Large$\circ$}}\newcommand{\opensqr}{\mbox{$\square$}}\newcommand{\opentri}{\mbox{$\triangle$}}
\newcommand{\opentridown}{\mbox{$\bigtriangledown$}}\newcommand{\fullcirc}{\mbox{{\Large$\bullet$}}}
\newcommand{\fullsqr}{\,\vrule height5pt depth0pt width5pt}
\newcommand{\dotted}{\protect\mbox{${\mathinner{\cdotp\cdotp\cdotp\cdotp\cdotp\cdotp}}$}}
\newcommand{\dashed}{\protect\mbox{-\ -\ -\ -}}\newcommand{\broken}{\protect\mbox{-- -- --}}
\newcommand{\longbroken}{\protect\mbox{--- --- ---}}\newcommand{\chain}{\protect\mbox{--- $\cdot$ ---}}
\newcommand{\dashddot}{\protect\mbox{--- $\cdot$ $\cdot$ ---}}\newcommand{\full}{\protect\mbox{------}}
\let\rmGamma\Gamma\let\rmDelta\Delta\let\rmTheta\Theta\let\rmLambda\Lambda\let\rmXi\Xi\let\rmPi\Pi
\let\rmSigma\Sigma\let\rmUpsilon\Upsilon\let\rmPhi\Phi\let\rmPsi\Psi\let\rmOmega\Omega"
    );
    DefMacro!("\\aunote{}", "\\thanks{#1}");
    DefMacro!(
      "\\corresau{}",
      "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}"
    );
  }

  // Every version's (85 of 85 copies; 1702.03412's lines): the subequation number "(2.1\textit{a})" (:649) and
  // `\returnthesubequation`, which re-establishes it after a paper's own `{subeqnarray}` numbering (:641-642;
  // cond-mat0107563, cond-mat0306566; 14 papers), `\useAMSsubequations` (:624-625; 2304.12657, 2601.23025,
  // 2608.01542), the title page's `\norule` and `\followon` and the footnote's `\fulloutfootnote` (:1066-1070,
  // :733-734), the class's `\indexsize` (:136) and `\boldit` (:166-167) and the theorem spacing `\halflineskip`
  // (:217-218).
  RawTeX!(
    r"\renewcommand\thesubequation{\theequation{\itshape\@alph\c@subequation}}
\newcommand\returnthesubequation{\gdef\thesubequation{\theequation{\itshape\@alph\c@subequation}}}
\newif\ifuseAMSsubequ\newcommand\useAMSsubequations{\global\useAMSsubequtrue}
\newif\iffollowon\newif\ifrmrule\newcommand\followon{\global\followontrue}\newcommand\norule{\global\rmruletrue}
\newif\iffofootnote\newcommand\fulloutfootnote{\global\fofootnotetrue}
\newcommand\indexsize{\@setfontsize\indexsize\@viiipt\@ixpt}
\newcommand\boldit{\usefont{OT1}{cmr}{bx}{it}\fontsize{10}{\f@baselineskip}\selectfont}
\newlength\halflineskip\setlength\halflineskip{6\p@ \@plus .5\p@ \@minus .5\p@}"
  );
  // Every version numbers the equations within the sections, "(2.1)" (v1.23-v1.28, cond-mat0107563's, 2608.04345's and
  // 2103.10260's copies: :615 `\renewcommand\theequation{\thesection.\@arabic\c@equation}`, `\section` itself zeroing
  // the counter, :301-306; the FLM copies, 2604.22963's and 2507.06135's JFM-FLM_Au.cls: :1317-1318, the same
  // `\theequation` and `\@addtoreset{equation}{section}`), and the appendices "(A 1)" (`\appendix`, :390-397 and
  // :616-626: the section counter and the equations zeroed, `\theequation{\thesection\,\@arabic\c@equation}`). The older
  // copies' `\nosectioneqnreset` (:297-299) numbers them through the paper, "(1)", the appendices' still reset (:303
  // `\ifappendix \setcounter{equation}\z@\fi`). The binding's article base kept "(1)", "(2)" (2608.04345: 116 equation
  // references; SYNC row 76; KNOWN_PERL_ERRORS #1097, Perl has no jfm binding). Guard
  // `perfect_kernel_batch78q::jfm_equations_within_sections`.
  RawTeX!(
    r"\renewcommand\theequation{\thesection.\@arabic\c@equation}\@addtoreset{equation}{section}
\def\appendix{\@appendix\setcounter{section}\z@\setcounter{equation}\z@\@addtoreset{equation}{section}%
\renewcommand\theequation{\thesection\,\@arabic\c@equation}\global\appendixtrue}"
  );
  {
    let _older = OriginGuard::new(DefinitionOrigin::Fallback);
    RawTeX!(
      r"\newif\ifno@eqnreset
\newcommand\nosectioneqnreset{\global\no@eqnresettrue\renewcommand\theequation{\@arabic\c@equation}%
\@removefromreset{equation}{section}}"
    );
  }
  // jfm.cls:299-306 (v1.23-v1.28, 72 of 74 census copies; 2608.04345's): `\section` itself zeroes the equation counter,
  // a starred one too ("(2.1)" after `\section*`, where the reset by the section counter gave "(2.3)"), only in the
  // appendices under `\nosectioneqnreset`. The FLM copies' `\section` (JFM-FLM_Au.cls:501, the zeroing commented out
  // at :481-487; 2507.06135 — 2505.15463's declares `JFM-FLM_Au` and is read raw) is a bare `\@startsection`, their equations reset by
  // `\@addtoreset{equation}{section}` (:1318), numbered sections only: the paper's copy decides. Residual (review
  // rev78q2): the appendices' `\section` is the kernel's `\@@appendix`, which the binding cannot wrap there (the locked
  // `\section` ignores a redefinition made in the document), so an appendix `\section*` keeps the count, "A 2" where
  // jfm.cls:303 zeroes it, "A 1". Guard `perfect_kernel_batch78q::jfm_section_star_zeroes_equations`.
  if shipped.as_deref().is_none_or(|text| {
    !crate::shipped_class::class_code_contains(text, "\\@addtoreset{equation}{section}")
  }) {
    RawTeX!(
      r"\let\lx@jfm@section\section
\def\section{\ifno@eqnreset\ifappendix\setcounter{equation}\z@\fi\else\setcounter{equation}\z@\fi\lx@jfm@section}"
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

/// Does this copy of jfm.cls set `\le`/`\ge` to the slanted ⩽ ⩾ when it finds the AMS symbol fonts (an uncommented
/// `\let\le=\leqslant`, 2103.10260's :1361)?
fn slants_inequalities(class: &str) -> bool {
  class.lines().any(|line| {
    line
      .split('%')
      .next()
      .unwrap_or_default()
      .contains("\\let\\le=\\leqslant")
  })
}
