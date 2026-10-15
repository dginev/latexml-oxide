//! Binding for the INFORMS journal classes: informs3 (and its informs3h, informs3a, informs3noheader copies), informs4
//! (and informs4a, informs4post, informs4Preprint) and informs5.
//!
//! The informs* classes are used by Operations Research, Management Science and the other INFORMS journals. A paper
//! ships its own copy, which falls back here by version (`informs4`) or by prefix (`informs3noheader`). The class's
//! front-matter API (\TITLE, \ARTICLEAUTHORS, \AUTHOR, \AFF, \ABSTRACT, \KEYWORDS, …), its float commands and its
//! theorem sets are mapped onto the frontmatter API and the kernel's floats.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  // informs3.cls:12 (informs4.cls:6, every copy) `\RequirePackage{amsmath,amssymb,ifthen,url,graphicx,color,array,
  // theorem}`: theorem.sty, not amsthm, so a paper's own `\usepackage{amsthm}` is a real load, after the class's
  // `\proof` (below), which it keeps, or after the paper cleared it (`\let\proof\relax\let\endproof\relax`), where it
  // puts amsthm's in its place (2008.02211, informs3opt.cls; `{proof}` was undefined there with amsthm preloaded).
  RequirePackage!("amssymb");
  RequirePackage!("theorem");

  // The class asked for (`fallback_request`, recorded by the versioned and the prefix fallback). The informs3 family
  // (informs3.cls, informs3h, informs3a, informs3noheader) declares twelve journals and no anonymous-review options
  // (informs3.cls:374-400), words its blind-review notice "(Authors' names blinded for peer review)" (:1034-1036) and, for
  // MOOR, prints the acknowledgment even under blind review (:1266-1274); informs3noheader prints no notice
  // (2609.04127 informs3noheader.cls:1032-1035). Older informs4 copies without `dblanonrev` word the notice as
  // informs3 does (2609.10587, 17368, 25924, 28947, 28263); the binding gives them the current wording.
  let request = lookup_string("fallback_request");
  let class = request.rsplit('/').next().unwrap_or_default();
  let informs3 = class.starts_with("informs3");
  let blind_notice = if class.starts_with("informs3noheader") {
    ""
  } else if informs3 {
    "(Authors' names blinded for peer review)"
  } else {
    "(Authors' names are not included for peer review)"
  };

  // informs4.cls:9-54 (informs3.cls declares twelve of these journals): the journal a paper is set for and the review
  // mode, as the switches a paper's own preamble tests (2609.37380: `\if@IJOC`, `\if@MOOR`, `\if@STSY`, `\if@IJOO`).
  RawTeX!(
    r"\newif\if@DECA\newif\if@IJOC\newif\if@INTE\newif\if@ISRE\newif\if@MNSC\newif\if@MKSC\newif\if@MOOR\newif\if@MSOM\newif\if@OPRE\newif\if@ORSC\newif\if@TRSC\newif\if@ITED\newif\if@SERV\newif\if@STSC\newif\if@STSY\newif\if@IJDS\newif\if@IJOO\newif\if@COPYEDIT\newif\if@BLINDREV\newif\if@NONBLINDREV"
  );
  DeclareOption!("deca", "\\@DECAtrue");
  DeclareOption!("ijoc", "\\@IJOCtrue");
  DeclareOption!("inte", "\\@INTEtrue");
  DeclareOption!("isre", "\\@ISREtrue");
  DeclareOption!("mnsc", "\\@MNSCtrue");
  DeclareOption!("mksc", "\\@MKSCtrue");
  DeclareOption!("moor", "\\@MOORtrue");
  DeclareOption!("msom", "\\@MSOMtrue");
  DeclareOption!("opre", "\\@OPREtrue");
  DeclareOption!("orsc", "\\@ORSCtrue");
  DeclareOption!("trsc", "\\@TRSCtrue");
  DeclareOption!("ited", "\\@ITEDtrue");
  DeclareOption!("copyedit", "\\@COPYEDITtrue");
  DeclareOption!("blindrev", "\\@BLINDREVtrue");
  DeclareOption!("nonblindrev", "\\@NONBLINDREVtrue");
  if !informs3 {
    DeclareOption!("stsy", "\\@STSYtrue");
    DeclareOption!("ijaa", "\\@INTEtrue");
    DeclareOption!("serv", "\\@SERVtrue");
    DeclareOption!("stsc", "\\@STSCtrue");
    DeclareOption!("ijds", "\\@IJDStrue");
    DeclareOption!("ijoo", "\\@IJOOtrue");
    DeclareOption!("dblanonrev", "\\@BLINDREVtrue");
    DeclareOption!("sglanonrev", "\\@NONBLINDREVtrue");
  }
  ProcessOptions!();

  // informs4.cls:948-985: the journal's metadata and the running heads and feet, each setter storing its value in
  // `\the…`; the class prints them only in the masthead and the page heads (page furniture), so they reach no element
  // here, but a paper reads them back (2609.23739, 38842: `\theRUNTITLE`; 22690, 24605, 25924: `\RRHSecondLine`;
  // 10587: `\JOURNAL`; 28084: `\theRRHFirstLine` in its own page head). The running-head lines default empty here;
  // the class's defaults (:1067-1083) name the journal and the running author and title.
  RawTeX!(
    r"\def\JOURNAL#1{\gdef\theJOURNAL{#1}}\def\theJOURNAL{Journal Name}
\def\JOURNALshort#1{\gdef\theJOURNALshort{#1}}\def\theJOURNALshort{Journal Name}
\def\VOLUME#1{\gdef\theVOLUME{#1}}\def\theVOLUME{00}
\def\NO#1{\gdef\theNO{#1}}\def\theNO{0}
\def\ISSUE#1{\gdef\theISSUE{#1}}\def\theISSUE{0000}
\def\MONTH#1{\gdef\theMONTH{#1}}\def\theMONTH{Xxxxx}
\def\YEAR#1{\gdef\theYEAR{#1}}\def\theYEAR{0000}
\def\ISSN#1{\gdef\theISSN{#1}}\def\theISSN{0000-0000}
\def\EISSN#1{\gdef\theEISSN{#1}}\def\theEISSN{0000-0000}
\def\DOI#1{\gdef\theDOI{#1}}\def\theDOI{10.1287/xxxx.0000.0000}
\def\FIRSTPAGE#1{\gdef\theFIRSTPAGE{#1}}\def\theFIRSTPAGE{000}
\def\LONGFIRSTPAGE#1{\gdef\theLONGFIRSTPAGE{#1}}\def\theLONGFIRSTPAGE{0001}
\def\LASTPAGE#1{\gdef\theLASTPAGE{#1}}\def\theLASTPAGE{000}
\def\SHORTYEAR#1{\gdef\theSHORTYEAR{#1}}\def\theSHORTYEAR{00}
\def\PAGERANGE#1{\gdef\thePAGERANGE{#1}}\def\thePAGERANGE{}
\def\theRUNAUTHOR{Author}\def\theRUNTITLE{Article Short Title}
\def\theECRUNAUTHOR{\theRUNAUTHOR}\def\theECRUNTITLE{\theRUNTITLE}
\def\ECTYPE#1{\gdef\theECTYPE{#1}}\def\theECTYPE{e-companion to\enspace}
\def\ECAUpunct#1{\gdef\theECAUpunct{#1}}\def\theECAUpunct{:\space}
\def\LRHFirstLine#1{\gdef\theLRHFirstLine{#1}}\def\theLRHFirstLine{}
\def\LRHSecondLine#1{\gdef\theLRHSecondLine{#1}}\def\theLRHSecondLine{}
\def\RRHFirstLine#1{\gdef\theRRHFirstLine{#1}}\def\theRRHFirstLine{}
\def\RRHSecondLine#1{\gdef\theRRHSecondLine{#1}}\def\theRRHSecondLine{}
\def\ECLRHFirstLine#1{\gdef\theECLRHFirstLine{#1}}\def\theECLRHFirstLine{}
\def\ECLRHSecondLine#1{\gdef\theECLRHSecondLine{#1}}\def\theECLRHSecondLine{}
\def\ECRRHFirstLine#1{\gdef\theECRRHFirstLine{#1}}\def\theECRRHFirstLine{}
\def\ECRRHSecondLine#1{\gdef\theECRRHSecondLine{#1}}\def\theECRRHSecondLine{}
\def\LRFFirstLine#1{\gdef\theLRFFirstLine{#1}}\def\theLRFFirstLine{}
\def\LRFSecondLine#1{\gdef\theLRFSecondLine{#1}}\def\theLRFSecondLine{}
\def\RRFFirstLine#1{\gdef\theRRFFirstLine{#1}}\def\theRRFFirstLine{}
\def\RRFSecondLine#1{\gdef\theRRFSecondLine{#1}}\def\theRRFSecondLine{}"
  );
  // informs4.cls:889-890 (2609.10587, 11171, 18126, 21433, 28859, 37380, 37489 use them undefined).
  RawTeX!(r"\def\argmax{\mathop{\rm arg\,max}}\def\argmin{\mathop{\rm arg\,min}}");

  // informs3.cls:98-247 (informs4.cls:145-294, the same in every version sampled — informs1 2005-2012, informs2,
  // informs3, informs4 2015-2024): bold letters and digits from the class's symbol fonts — `boperators` (OT1 cmr bx n,
  // bold upright: `\BFx`, `\BFzero`, `\BFGamma`), `bletters` (OML cmm b it, bold italic Greek: `\BFbeta`) and `bsymbols`
  // (OMS cmsy b n, bold calligraphic: `\BFcalX`, `\BFcal`) — as `\mathbf` and `\boldsymbol` here, which keep their
  // weight (a `\DeclareMathSymbol` keeps only the glyph; `\BFx` 13 of the 75 sampled papers, `\BFy` 11, `\BFbeta` 7:
  // 1501.01126, 1710.02642, 2008.02154, 2010.06802).
  RawTeX!(
    r"\def\BFa{\mathbf{a}}\def\BFb{\mathbf{b}}\def\BFc{\mathbf{c}}\def\BFd{\mathbf{d}}\def\BFe{\mathbf{e}}
\def\BFf{\mathbf{f}}\def\BFg{\mathbf{g}}\def\BFh{\mathbf{h}}\def\BFi{\mathbf{i}}\def\BFj{\mathbf{j}}
\def\BFk{\mathbf{k}}\def\BFl{\mathbf{l}}\def\BFm{\mathbf{m}}\def\BFn{\mathbf{n}}\def\BFo{\mathbf{o}}
\def\BFp{\mathbf{p}}\def\BFq{\mathbf{q}}\def\BFr{\mathbf{r}}\def\BFs{\mathbf{s}}\def\BFt{\mathbf{t}}
\def\BFu{\mathbf{u}}\def\BFv{\mathbf{v}}\def\BFw{\mathbf{w}}\def\BFx{\mathbf{x}}\def\BFy{\mathbf{y}}
\def\BFz{\mathbf{z}}\def\BFA{\mathbf{A}}\def\BFB{\mathbf{B}}\def\BFC{\mathbf{C}}\def\BFD{\mathbf{D}}
\def\BFE{\mathbf{E}}\def\BFF{\mathbf{F}}\def\BFG{\mathbf{G}}\def\BFH{\mathbf{H}}\def\BFI{\mathbf{I}}
\def\BFJ{\mathbf{J}}\def\BFK{\mathbf{K}}\def\BFL{\mathbf{L}}\def\BFM{\mathbf{M}}\def\BFN{\mathbf{N}}
\def\BFO{\mathbf{O}}\def\BFP{\mathbf{P}}\def\BFQ{\mathbf{Q}}\def\BFR{\mathbf{R}}\def\BFS{\mathbf{S}}
\def\BFT{\mathbf{T}}\def\BFU{\mathbf{U}}\def\BFV{\mathbf{V}}\def\BFW{\mathbf{W}}\def\BFX{\mathbf{X}}
\def\BFY{\mathbf{Y}}\def\BFZ{\mathbf{Z}}\def\BFzero{\mathbf{0}}\def\BFone{\mathbf{1}}\def\BFtwo{\mathbf{2}}
\def\BFthree{\mathbf{3}}\def\BFfour{\mathbf{4}}\def\BFfive{\mathbf{5}}\def\BFsix{\mathbf{6}}\def\BFseven{\mathbf{7}}
\def\BFeight{\mathbf{8}}\def\BFnine{\mathbf{9}}\def\BFalpha{\boldsymbol{\alpha}}\def\BFbeta{\boldsymbol{\beta}}\def\BFgamma{\boldsymbol{\gamma}}
\def\BFdelta{\boldsymbol{\delta}}\def\BFepsilon{\boldsymbol{\epsilon}}\def\BFzeta{\boldsymbol{\zeta}}\def\BFeta{\boldsymbol{\eta}}\def\BFtheta{\boldsymbol{\theta}}
\def\BFiota{\boldsymbol{\iota}}\def\BFkappa{\boldsymbol{\kappa}}\def\BFlambda{\boldsymbol{\lambda}}\def\BFmu{\boldsymbol{\mu}}\def\BFnu{\boldsymbol{\nu}}
\def\BFxi{\boldsymbol{\xi}}\def\BFpi{\boldsymbol{\pi}}\def\BFrho{\boldsymbol{\rho}}\def\BFsigma{\boldsymbol{\sigma}}\def\BFtau{\boldsymbol{\tau}}
\def\BFupsilon{\boldsymbol{\upsilon}}\def\BFphi{\boldsymbol{\phi}}\def\BFchi{\boldsymbol{\chi}}\def\BFpsi{\boldsymbol{\psi}}\def\BFomega{\boldsymbol{\omega}}
\def\BFvarepsilon{\boldsymbol{\varepsilon}}\def\BFvartheta{\boldsymbol{\vartheta}}\def\BFvarpi{\boldsymbol{\varpi}}\def\BFvarrho{\boldsymbol{\varrho}}\def\BFvarsigma{\boldsymbol{\varsigma}}
\def\BFvarphi{\boldsymbol{\varphi}}\def\BFGamma{\mathbf{\Gamma}}\def\BFDelta{\mathbf{\Delta}}\def\BFTheta{\mathbf{\Theta}}\def\BFLambda{\mathbf{\Lambda}}
\def\BFXi{\mathbf{\Xi}}\def\BFPi{\mathbf{\Pi}}\def\BFSigma{\mathbf{\Sigma}}\def\BFUpsilon{\mathbf{\Upsilon}}\def\BFPhi{\mathbf{\Phi}}
\def\BFPsi{\mathbf{\Psi}}\def\BFOmega{\mathbf{\Omega}}\def\BFimath{\boldsymbol{\imath}}\def\BFjmath{\boldsymbol{\jmath}}\def\BFell{\boldsymbol{\ell}}
\def\BFwp{\boldsymbol{\wp}}\def\BFnabla{\boldsymbol{\nabla}}\def\BFcalA{\boldsymbol{\mathcal{A}}}\def\BFcalB{\boldsymbol{\mathcal{B}}}\def\BFcalC{\boldsymbol{\mathcal{C}}}
\def\BFcalD{\boldsymbol{\mathcal{D}}}\def\BFcalE{\boldsymbol{\mathcal{E}}}\def\BFcalF{\boldsymbol{\mathcal{F}}}\def\BFcalG{\boldsymbol{\mathcal{G}}}\def\BFcalH{\boldsymbol{\mathcal{H}}}
\def\BFcalI{\boldsymbol{\mathcal{I}}}\def\BFcalJ{\boldsymbol{\mathcal{J}}}\def\BFcalK{\boldsymbol{\mathcal{K}}}\def\BFcalL{\boldsymbol{\mathcal{L}}}\def\BFcalM{\boldsymbol{\mathcal{M}}}
\def\BFcalN{\boldsymbol{\mathcal{N}}}\def\BFcalO{\boldsymbol{\mathcal{O}}}\def\BFcalP{\boldsymbol{\mathcal{P}}}\def\BFcalQ{\boldsymbol{\mathcal{Q}}}\def\BFcalR{\boldsymbol{\mathcal{R}}}
\def\BFcalS{\boldsymbol{\mathcal{S}}}\def\BFcalT{\boldsymbol{\mathcal{T}}}\def\BFcalU{\boldsymbol{\mathcal{U}}}\def\BFcalV{\boldsymbol{\mathcal{V}}}\def\BFcalW{\boldsymbol{\mathcal{W}}}
\def\BFcalX{\boldsymbol{\mathcal{X}}}\def\BFcalY{\boldsymbol{\mathcal{Y}}}\def\BFcalZ{\boldsymbol{\mathcal{Z}}}
\def\BFcal#1{\boldsymbol{\mathcal{#1}}}"
  );
  // informs4.cls:391, 638, 867, 883, 913, 1471-1472 (informs3.cls:426, 931, 1506-1507): `\fs.size.skip.` sets the type
  // size, `\HD{h}{d}` an invisible strut that `\up`/`\down`/`\updown` put in a table row (6, 5, 3 papers: 1402.4593,
  // 1810.10900), `\halmos` the end-of-proof box (9 papers: 1011.5677), `\n` a `\nobreak` (1510.08335).
  RawTeX!(
    r"\def\fs.#1.#2.{\fontsize{#1}{#2}\selectfont}
\def\HD#1#2{\vrule height #1pt depth #2pt width 0pt\relax}
\def\up{\HD{13}{0}}\def\down{\HD{0}{5}}\def\updown{\up\down}
\def\halmos{\mbox{\quad$\square$}}
\def\n{\nobreak}"
  );
  // informs4.cls:1001-1020 (informs3.cls:1047-1063): the subject classifications the title page lists under the
  // keywords (`\MSCCLASS` and `\ORMSCLASS` 6 papers, `\AREAOFREVIEW` 4, `\SUBJECTCLASS` 3; 1009.2653, 1508.02966,
  // 0908.2788), headed by the class's names (`\MSCCLASSname`, … which a paper may redefine for the print; the
  // classification's name here is their text).
  RawTeX!(
    r"\def\SUBJECTCLASSname{{\it Subject classifications\/}{\kern0.7pt}:\enskip}
\def\AREAOFREVIEWname{{\it Area of review\/}{\kern0.7pt}:\enskip}
\def\MSCCLASSname{{\it MSC2000 subject classification\/}{\kern0.7pt}:\enskip}
\def\ORMSCLASSname{{\it OR/MS subject classification\/}{\kern0.7pt}:\enskip}
\def\SUBJECTCLASS#1{\lx@add@classification[scheme=subject,name={Subject classifications:~}]{#1}}
\def\AREAOFREVIEW#1{\lx@add@classification[scheme=area of review,name={Area of review:~}]{#1}}
\def\MSCCLASS#1{\lx@add@classification[scheme=MSC,name={MSC2000 subject classification:~}]{#1}}
\def\ORMSCLASS#1{\lx@add@classification[scheme=OR/MS,name={OR/MS subject classification:~}]{#1}}"
  );
  // informs4.cls:1377-1387 (informs3.cls:1412-1422): `{Trivlist}`, a trivlist without the space around it (3 papers:
  // 1408.4711), on which the class's `\proof` is built; the spacing is print layout.
  RawTeX!(r"\let\Trivlist\trivlist\let\endTrivlist\endtrivlist");
  // informs4.cls:1829-1830 (informs3.cls:1864-1865) `\def\proof#1{\Trivlist\item[\hspace*{1em}\hskip\labelsep{\it #1
  // \enskip }]…}`: the proof's head is its argument, in italics, as written (`\proof{Proof.}`, `\begin{proof}{Proof of
  // Theorem 1.}` — 229 and 87 of the 456 inventory papers), and no end mark (the papers write `\Halmos`). amsthm's
  // proof took the argument as its first text: "Proof. Proof." (1301.2609). A constructor pair, as autart's `{pf}`
  // (autart_cls.rs): the body stays in the mode around it.
  RawTeX!(r"\def\proof#1{\lx@informs@proof{{\itshape #1}}}");
  DefConstructor!(
    "\\lx@informs@proof{}",
    "<ltx:proof><ltx:title>#1</ltx:title>"
  );
  DefConstructor!("\\endproof", sub[document, _args] {
    document.maybe_close_element("ltx:proof")?;
  });
  // informs4.cls:1420-1452 (informs3.cls:1455-1487): `\NatBibNumeric` renews `{thebibliography}` to number natbib's
  // entries; the bibliography is the natbib binding's here, which numbers them already (8 papers: 1009.2653).
  def_macro_noop("\\NatBibNumeric")?;
  // informs4.cls:1621-1638 (informs3.cls the same): `{descr}`, a description list with an upright label (1501.01126).
  RawTeX!(
    r"\newcommand*\descrlabel[1]{\hspace\labelsep\normalfont #1}
\def\descr{\ifnum \@itemdepth >\thr@@\@toodeep\else\advance\@itemdepth\@ne\list{}{\DescrListParams}\fi}
\def\DescrListParams{\leftmargin\z@ \labelwidth\z@\ifnum \@itemdepth=1\itemindent1em\else  \ifnum \@itemdepth=2\itemindent2em\else \itemindent\labelsep\advance\itemindent3em \fi\fi\let\makelabel\descrlabel}
\let\enddescr=\endlist"
  );
  // informs4.cls:1173-1175, 1822, 1853, 2141-2147 (informs3.cls:1857): the fonts and hooks a paper redefines for its
  // title page, theorem heads and float notes (`\TITLEfont` 2311.06745, `\TheoremHeaderFont` 2101.01816, `\HOOKtop`
  // 2503.15150, `\FigureNoteStyle` 2112.05010).
  RawTeX!(
    r"\def\TITLEfont{\rm}\def\AUTHORfont{\rm}\def\AFFfont{\rm}\def\HOOKtop{\vspace*{0pt}}
\def\FigureNoteStyle{\rm}\def\TableNoteStyle{\rm}
\def\thit{\it}\def\TheoremTextFont{\thit}\def\ExampleHeaderFont{\mdseries\scshape}\def\ExampleTextFont{\rm}"
  );
  if informs3 {
    RawTeX!(r"\def\TheoremHeaderFont{\mdseries\scshape}");
  } else {
    RawTeX!(r"\def\TheoremHeaderFont{\sffamily\mdseries\scshape}");
  }
  // informs4.cls:2742-2744 (informs3.cls the same): `\ECDisclaimer`, the production notice "This page is intentionally
  // blank…" on the e-companion's title page — page furniture (3 papers: 1102.2551).
  def_macro_noop("\\ECDisclaimer")?;

  // Frontmatter / paper metadata — preserve author content.
  // \TITLE / \ARTICLETITLE → \title to populate the document title.
  DefMacro!("\\TITLE{}", "\\title{#1}");
  DefMacro!("\\ARTICLETITLE{}", "\\title{#1}");
  // Running header variants — short title for header; preserve as note.
  // Under the blind-review options the running heads print the class's notice, not the running authors
  // (informs4.cls:1071-1076, informs3.cls:1108-1111); the e-companion heads print `\theECRUNAUTHOR` either way
  // (:1067-1069).
  DefMacro!(
    "\\RUNAUTHOR{}",
    "\\gdef\\theRUNAUTHOR{#1}\\if@BLINDREV\\expandafter\\@gobble\\else\\expandafter\\lx@informs@runningauthor\\fi{#1}"
  );
  DefMacro!(
    "\\lx@informs@runningauthor{}",
    "\\@add@frontmatter{ltx:note}[role=runningauthor]{#1}"
  );
  DefMacro!(
    "\\RUNTITLE{}",
    "\\gdef\\theRUNTITLE{#1}\\@add@frontmatter{ltx:note}[role=runningtitle]{#1}"
  );
  DefMacro!(
    "\\ECRUNAUTHOR{}",
    "\\gdef\\theECRUNAUTHOR{#1}\\@add@frontmatter{ltx:note}[role=ec-runningauthor]{#1}"
  );
  DefMacro!(
    "\\ECRUNTITLE{}",
    "\\gdef\\theECRUNTITLE{#1}\\@add@frontmatter{ltx:note}[role=ec-runningtitle]{#1}"
  );
  // informs4.cls:1198-1206 (informs3.cls:1236-1244): `\AUTHOR{name}` and `\AFF{affiliation}` each take one argument,
  // the affiliation under its author. They were read as `\AUTHOR{name}{affiliation}` and `\AFF[]{…}`, so `\AUTHOR`
  // took the following `\AFF` token as its affiliation and the affiliations became loose notes (2609.37380). Each
  // `\AUTHOR` adds its authors to the ones before, and the superscript marks of a marked author list link the marked
  // `\AFF` lines to their authors (2609.17368 `$^{a,e}$`, 22690 and 28084 `\textsuperscript{1}`).
  DefMacro!("\\AUTHOR{}", "\\lx@add@authors@list{#1}");
  DefMacro!("\\AFF{}", "\\lx@add@affiliation@marked{#1}");
  // \ABSTRACT → abstract env so the text is preserved as document abstract.
  DefMacro!("\\ABSTRACT{}", "\\begin{abstract}#1\\end{abstract}");
  DefMacro!("\\ARTICLEABSTRACT{}", "\\begin{abstract}#1\\end{abstract}");
  DefMacro!(
    "\\KEYWORDS{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}"
  );
  DefMacro!(
    "\\MANUSCRIPTNO{}",
    "\\@add@frontmatter{ltx:note}[role=manuscriptno]{#1}"
  );
  // informs4.cls:1952 (informs3.cls:1946): the title page prints the history unless under blind review (2609.08001,
  // 21433, 28084).
  DefMacro!(
    "\\HISTORY{}",
    "\\if@BLINDREV\\expandafter\\@gobble\\else\\expandafter\\lx@informs@history\\fi{#1}"
  );
  DefMacro!(
    "\\lx@informs@history{}",
    "\\@add@frontmatter{ltx:note}[role=history]{#1}"
  );
  // informs4.cls:987-995, 1225-1233 (informs3.cls:1032-1039, 1266-1274): when it loads, the class decides that under
  // the blind-review options the author block prints only its notice in place of the names (none in informs3noheader)
  // and `\ACKNOWLEDGMENT` and `\AUTHORBIO` print nothing; informs3's MOOR acknowledgment, defined after, prints anyway.
  // No 2609 paper sets blindrev or dblanonrev (2609.23739 has it commented out).
  raw_tex(&format!(r"\def\lx@informs@blindnotice{{{blind_notice}}}"))?;
  RawTeX!(
    r"\long\def\ARTICLEAUTHORS#1{#1}\long\def\ACKNOWLEDGMENT#1{\lx@informs@acknowledgment{#1}}\long\def\AUTHORBIO#1{#1}
\if@BLINDREV\long\def\ARTICLEAUTHORS#1{\ifx\lx@informs@blindnotice\@empty\else\@add@frontmatter{ltx:note}[role=authors]{\lx@informs@blindnotice}\fi}\long\def\ACKNOWLEDGMENT#1{}\long\def\AUTHORBIO#1{}\fi"
  );
  if informs3 {
    RawTeX!(r"\if@MOOR\long\def\ACKNOWLEDGMENT#1{\lx@informs@acknowledgment{#1}}\fi");
  }
  // informs4.cls:1023: the funding statement the title page prints ("Funding: …"; 2609.17368).
  DefMacro!(
    "\\FUNDING{}",
    "\\@add@frontmatter{ltx:note}[role=funding]{#1}"
  );
  DefMacro!(
    "\\authorinfo{}",
    "\\@add@frontmatter{ltx:note}[role=authorinfo]{#1}"
  );
  def_macro_noop("\\thetitle")?;

  // Layout / structure switches — no visual effect in XML.
  // The class's theorem styles TH and EX end the head with a period (informs3.cls:1809-1856 `\theorem@headerfont ##1\
  // ##2.`), which theorem.sty's own styles do not (`\thm@headpunct` empty, theorem_sty.rs), so each set gives it.
  // informs3.cls:1511-1527, 2089, 2117-2124, 2442-2523, 2727-2738: the theorem sets a paper picks (numbered through
  // or by section; the class's theorem.sty styles TH and EX, :1809-1856, are amsthm's plain and definition: an italic
  // and an upright body), the equation numbering, and the e-companion switch that renumbers sections,
  // equations, theorems, figures and tables `EC.n` — `{assumption}` and `\ECSwitch` were undefined (2609.08001).
  RawTeX!(
    r"\def\TheoremsNumberedThrough{\theoremstyle{plain}\thm@headpunct{.}\newtheorem{theorem}{Theorem}\newtheorem{lemma}{Lemma}\newtheorem{proposition}{Proposition}\newtheorem{corollary}{Corollary}\newtheorem{claim}{Claim}\newtheorem{conjecture}{Conjecture}\newtheorem{hypothesis}{Hypothesis}\newtheorem{assumption}{Assumption}\theoremstyle{definition}\thm@headpunct{.}\newtheorem{remark}{Remark}\newtheorem{example}{Example}\newtheorem{problem}{Problem}\newtheorem{definition}{Definition}\newtheorem{question}{Question}\newtheorem{answer}{Answer}\newtheorem{exercise}{Exercise}\def\ECHowTheorems{\setcounter{theorem}{0}\expandafter\let\csname oldthetheorem\expandafter\endcsname\csname thetheorem\endcsname\expandafter\def\csname thetheorem\endcsname{EC.\csname oldthetheorem\endcsname}\setcounter{lemma}{0}\expandafter\let\csname oldthelemma\expandafter\endcsname\csname thelemma\endcsname\expandafter\def\csname thelemma\endcsname{EC.\csname oldthelemma\endcsname}\setcounter{proposition}{0}\expandafter\let\csname oldtheproposition\expandafter\endcsname\csname theproposition\endcsname\expandafter\def\csname theproposition\endcsname{EC.\csname oldtheproposition\endcsname}\setcounter{corollary}{0}\expandafter\let\csname oldthecorollary\expandafter\endcsname\csname thecorollary\endcsname\expandafter\def\csname thecorollary\endcsname{EC.\csname oldthecorollary\endcsname}\setcounter{claim}{0}\expandafter\let\csname oldtheclaim\expandafter\endcsname\csname theclaim\endcsname\expandafter\def\csname theclaim\endcsname{EC.\csname oldtheclaim\endcsname}\setcounter{conjecture}{0}\expandafter\let\csname oldtheconjecture\expandafter\endcsname\csname theconjecture\endcsname\expandafter\def\csname theconjecture\endcsname{EC.\csname oldtheconjecture\endcsname}\setcounter{hypothesis}{0}\expandafter\let\csname oldthehypothesis\expandafter\endcsname\csname thehypothesis\endcsname\expandafter\def\csname thehypothesis\endcsname{EC.\csname oldthehypothesis\endcsname}\setcounter{assumption}{0}\expandafter\let\csname oldtheassumption\expandafter\endcsname\csname theassumption\endcsname\expandafter\def\csname theassumption\endcsname{EC.\csname oldtheassumption\endcsname}\setcounter{remark}{0}\expandafter\let\csname oldtheremark\expandafter\endcsname\csname theremark\endcsname\expandafter\def\csname theremark\endcsname{EC.\csname oldtheremark\endcsname}\setcounter{example}{0}\expandafter\let\csname oldtheexample\expandafter\endcsname\csname theexample\endcsname\expandafter\def\csname theexample\endcsname{EC.\csname oldtheexample\endcsname}\setcounter{problem}{0}\expandafter\let\csname oldtheproblem\expandafter\endcsname\csname theproblem\endcsname\expandafter\def\csname theproblem\endcsname{EC.\csname oldtheproblem\endcsname}\setcounter{definition}{0}\expandafter\let\csname oldthedefinition\expandafter\endcsname\csname thedefinition\endcsname\expandafter\def\csname thedefinition\endcsname{EC.\csname oldthedefinition\endcsname}\setcounter{question}{0}\expandafter\let\csname oldthequestion\expandafter\endcsname\csname thequestion\endcsname\expandafter\def\csname thequestion\endcsname{EC.\csname oldthequestion\endcsname}\setcounter{answer}{0}\expandafter\let\csname oldtheanswer\expandafter\endcsname\csname theanswer\endcsname\expandafter\def\csname theanswer\endcsname{EC.\csname oldtheanswer\endcsname}\setcounter{exercise}{0}\expandafter\let\csname oldtheexercise\expandafter\endcsname\csname theexercise\endcsname\expandafter\def\csname theexercise\endcsname{EC.\csname oldtheexercise\endcsname}}}
\def\TheoremsNumberedBySection{\theoremstyle{plain}\thm@headpunct{.}\newtheorem{theorem}{Theorem}[section]\newtheorem{lemma}{Lemma}[section]\newtheorem{proposition}{Proposition}[section]\newtheorem{corollary}{Corollary}[section]\newtheorem{claim}{Claim}[section]\newtheorem{conjecture}{Conjecture}[section]\newtheorem{hypothesis}{Hypothesis}[section]\newtheorem{assumption}{Assumption}[section]\theoremstyle{definition}\thm@headpunct{.}\newtheorem{remark}{Remark}[section]\newtheorem{example}{Example}[section]\newtheorem{problem}{Problem}[section]\newtheorem{definition}{Definition}[section]\newtheorem{question}{Question}[section]\newtheorem{answer}{Answer}[section]\newtheorem{exercise}{Exercise}[section]\def\ECHowTheorems{\setcounter{theorem}{0}\expandafter\let\csname oldthetheorem\expandafter\endcsname\csname thetheorem\endcsname\expandafter\def\csname thetheorem\endcsname{\csname oldthetheorem\endcsname}\setcounter{lemma}{0}\expandafter\let\csname oldthelemma\expandafter\endcsname\csname thelemma\endcsname\expandafter\def\csname thelemma\endcsname{\csname oldthelemma\endcsname}\setcounter{proposition}{0}\expandafter\let\csname oldtheproposition\expandafter\endcsname\csname theproposition\endcsname\expandafter\def\csname theproposition\endcsname{\csname oldtheproposition\endcsname}\setcounter{corollary}{0}\expandafter\let\csname oldthecorollary\expandafter\endcsname\csname thecorollary\endcsname\expandafter\def\csname thecorollary\endcsname{\csname oldthecorollary\endcsname}\setcounter{claim}{0}\expandafter\let\csname oldtheclaim\expandafter\endcsname\csname theclaim\endcsname\expandafter\def\csname theclaim\endcsname{\csname oldtheclaim\endcsname}\setcounter{conjecture}{0}\expandafter\let\csname oldtheconjecture\expandafter\endcsname\csname theconjecture\endcsname\expandafter\def\csname theconjecture\endcsname{\csname oldtheconjecture\endcsname}\setcounter{hypothesis}{0}\expandafter\let\csname oldthehypothesis\expandafter\endcsname\csname thehypothesis\endcsname\expandafter\def\csname thehypothesis\endcsname{\csname oldthehypothesis\endcsname}\setcounter{assumption}{0}\expandafter\let\csname oldtheassumption\expandafter\endcsname\csname theassumption\endcsname\expandafter\def\csname theassumption\endcsname{\csname oldtheassumption\endcsname}\setcounter{remark}{0}\expandafter\let\csname oldtheremark\expandafter\endcsname\csname theremark\endcsname\expandafter\def\csname theremark\endcsname{\csname oldtheremark\endcsname}\setcounter{example}{0}\expandafter\let\csname oldtheexample\expandafter\endcsname\csname theexample\endcsname\expandafter\def\csname theexample\endcsname{\csname oldtheexample\endcsname}\setcounter{problem}{0}\expandafter\let\csname oldtheproblem\expandafter\endcsname\csname theproblem\endcsname\expandafter\def\csname theproblem\endcsname{\csname oldtheproblem\endcsname}\setcounter{definition}{0}\expandafter\let\csname oldthedefinition\expandafter\endcsname\csname thedefinition\endcsname\expandafter\def\csname thedefinition\endcsname{\csname oldthedefinition\endcsname}\setcounter{question}{0}\expandafter\let\csname oldthequestion\expandafter\endcsname\csname thequestion\endcsname\expandafter\def\csname thequestion\endcsname{\csname oldthequestion\endcsname}\setcounter{answer}{0}\expandafter\let\csname oldtheanswer\expandafter\endcsname\csname theanswer\endcsname\expandafter\def\csname theanswer\endcsname{\csname oldtheanswer\endcsname}\setcounter{exercise}{0}\expandafter\let\csname oldtheexercise\expandafter\endcsname\csname theexercise\endcsname\expandafter\def\csname theexercise\endcsname{\csname oldtheexercise\endcsname}}}
\def\EquationsNumberedThrough{\def\theequation{\arabic{equation}}\def\ECHowEquations{\setcounter{equation}{0}\let\oldtheequation\theequation\def\theequation{EC.\oldtheequation}}}
\def\EquationsNumberedBySection{\@addtoreset{equation}{section}\def\theequation{\thesection.\arabic{equation}}\def\ECHowEquations{\setcounter{equation}{0}\let\oldtheequation\theequation\def\theequation{EC.\oldtheequation}}}
\EquationsNumberedThrough
\def\ECHowSections{\setcounter{section}{0}\renewcommand\thesection{EC.\@arabic\c@section}\renewcommand\thesubsection{\thesection.\@arabic\c@subsection}\renewcommand\thesubsubsection{\thesubsection.\@arabic\c@subsubsection}\renewcommand\theparagraph{\thesubsubsection.\@arabic\c@paragraph}\renewcommand\thesubparagraph{\theparagraph.\@arabic\c@subparagraph}}
\def\ECSwitch{\ECHowTheorems\ECHowEquations\ECHowSections\setcounter{figure}{0}\renewcommand\thefigure{EC.\@arabic\c@figure}\setcounter{table}{0}\renewcommand\thetable{EC.\@arabic\c@table}}
\def\ECHead#1{\par\noindent{\raggedright\bfseries #1\par}}"
  );
  // informs4.cls:2680-2696 (informs3.cls:2529-2545): the e-companion's repeated statements, unnumbered here by the
  // paper's own `\renewcommand\therepeattheorem` (`{repeattheorem}` 5 papers, `{repeatproposition}` 5, `{repeatlemma}` 3:
  // 1704.09011, 1707.00361, 1108.3299).
  RawTeX!(
    r"\def\ECRepeatTheorems{\theoremstyle{plain}\thm@headpunct{.}\newtheorem{repeattheorem}{Theorem}\newtheorem{repeatlemma}{Lemma}\newtheorem{repeatproposition}{Proposition}\newtheorem{repeatcorollary}{Corollary}\newtheorem{repeatclaim}{Claim}\newtheorem{repeatconjecture}{Conjecture}\newtheorem{repeathypothesis}{Hypothesis}\newtheorem{repeatassumption}{Assumption}\theoremstyle{definition}\thm@headpunct{.}\newtheorem{repeatremark}{Remark}\newtheorem{repeatexample}{Example}\newtheorem{repeatproblem}{Problem}\newtheorem{repeatdefinition}{Definition}\newtheorem{repeatquestion}{Question}\newtheorem{repeatanswer}{Answer}\newtheorem{repeatexercise}{Exercise}}"
  );
  // The line-spacing switches (informs4.cls:407-470, informs2.cls:475-485): `\SingleSpacedXII` (4 papers: 1909.08021),
  // `\TableSpaced` (3: 1905.08224), informs2's `\SingleSpaced`/`\DoubleSpaced` (3: 0910.0063).
  def_macro_noop("\\SingleSpacedXII")?;
  def_macro_noop("\\TableSpaced")?;
  def_macro_noop("\\SingleSpaced")?;
  def_macro_noop("\\DoubleSpaced")?;
  def_macro_noop("\\OneAndAHalfSpacedXI")?;
  def_macro_noop("\\OneAndAHalfSpacedXII")?;
  def_macro_noop("\\DoubleSpacedXI")?;
  def_macro_noop("\\DoubleSpacedXII")?;
  def_macro_noop("\\SingleSpacedXI")?;

  // informs4.cls:1702-1763 (informs3.cls the same): `{henumerate}` and `{hitemize}`, enumerate and itemize with a
  // hanging layout (2609.21433).
  DefMacro!(T_CS!("\\begin{henumerate}"), None, "\\begin{enumerate}");
  DefMacro!(T_CS!("\\end{henumerate}"), None, "\\end{enumerate}");
  DefMacro!(T_CS!("\\begin{hitemize}"), None, "\\begin{itemize}");
  DefMacro!(T_CS!("\\end{hitemize}"), None, "\\end{itemize}");

  // {APPENDICES} env — render contents as appendix section.
  DefMacro!(T_CS!("\\begin{APPENDICES}"), None, "\\appendix");
  DefMacro!(T_CS!("\\end{APPENDICES}"), None, "");

  // informs3.cls L932: `\def\Halmos{\mbox{\quad$\square$}}` — proof-end
  // QED box. Render the square in math mode.
  DefMacro!("\\Halmos", "\\ensuremath{\\square}");
  // informs3.cls L1231: `\def\EMAIL#1{#1}` — used within \AFF; plain
  // passthrough of the email text.
  DefMacro!("\\EMAIL{}", "#1");
  // informs3.cls L1273: `\long\def\ACKNOWLEDGMENT#1{\section*{\bf
  // \theACKname.}{#1}}` (\theACKname defaults to "Acknowledgments").
  // Route the body to a structural acknowledgements block (see
  // feedback: prefer ltx:acknowledgements over a flattened \section*);
  // `\ACKNOWLEDGMENT` and `\AUTHORBIO` (2609.24605) are defined with the blind-review choice above.
  DefConstructor!(
    "\\lx@informs@acknowledgment{}",
    "<ltx:acknowledgements name='Acknowledgments'>#1</ltx:acknowledgements>"
  );
  // \ACKname{name} sets the acknowledgements heading name.
  def_macro_noop("\\ACKname{}")?;
  // informs3.cls L2642: `\newenvironment{APPENDIX}[1]{…appendix with
  // title #1…}`. The singular env wraps a single titled appendix.
  // Begin enters appendix mode + emits the title as a section; the arg
  // is read by the helper. Mirrors the {APPENDICES} handling.
  DefMacro!(
    T_CS!("\\begin{APPENDIX}"),
    None,
    "\\appendix\\lx@informs@appendixhead"
  );
  DefMacro!("\\lx@informs@appendixhead{}", "\\section{#1}");
  DefMacro!(T_CS!("\\end{APPENDIX}"), None, "");

  // informs4.cls:2142-2148, 2229-2259, 2449-2470: `\FIGURE{body}{caption}{note}` and `\TABLE{caption}{body}{note}`
  // inside the float environments, the caption before the body and the note after it; the body is a group (:2251
  // `{#1}`, :2470 `{\TEGT #2}`), so its font changes stay in it; a figure's note is headed "Note.", a table's prints as
  // written; an empty note prints nothing (2609.10587, 25924). The class gives `\caption` an empty list entry; ours
  // takes the list entry from the caption (Perl latex_constructs.pool.ltxml:3189). The class's rotation and box
  // measurement are layout.
  RawTeX!(
    r"\def\FigureNoteName{Note.}\def\TableNoteName{Note.}
\long\def\FIGURE#1#2#3{\caption[]{#2}{#1}\long\def\testfignote{#3}\long\def\itis@empty{}\ifx\testfignote\itis@empty\else\par\noindent{\it\FigureNoteName}\enskip #3\par\fi}
\long\def\TABLE#1#2#3{\caption[]{#1}{#2}\long\def\testtabnote{#3}\long\def\itis@empty{}\ifx\testtabnote\itis@empty\else\par\noindent #3\par\fi}"
  );
});
