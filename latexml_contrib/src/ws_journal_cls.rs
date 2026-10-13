//! Stub for World Scientific journal classes (ws-ijgmmp, ws-ijmpd, etc.).
//!
//! These classes share frontmatter macros like `\catchline{vol}{no}{year}{pgs}{pgsE}`,
//! `\title{...}`, `\Author{name}`. Raw load of the cls may fail before
//! defining these (cls uses complex font/page-layout machinery that
//! doesn't transfer to HTML conversion). Stubs preserve substantive
//! content as frontmatter notes.
//!
//! Witness: 2306.12455 (ws-ijgmmp.cls), 2306.15982 (ws-ijmpd.cls).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("graphicx");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  // ws-ijmpe.cls:226-230 (ws-ijmpcs.cls:155, ws-mpla.cls:147, ws-rv9x6.cls:160 the same): `\def\bm#1{\mathchoice
  // {\mbox{\boldmath$#1$}}…}`, bold math the class defines itself, its argument in a box — 1308.3116, 2304.11343,
  // hep-ph0401137 (ws-procs9x6). A `\left` without its `\right` in the argument (`\bm{\left\langle} x
  // \bm{\right\rangle}`) is TeX's error at the box's end; it is set at the outer level with a Warning, as bm.sty
  // would set it (bm_sty.rs `define_grouped_bm`, OXIDIZED_DESIGN #544).
  bm_sty::define_grouped_bm()?;
  // The figure packages the ws journal and proceedings classes input (ws-ijmpa.cls:551-555, ws-ijmpe.cls:658-662,
  // ws-procs9x6.cls:614-618 `\InputIfFileExists{psfig.sty}…{epsfsafe.tex}…{epsfig.sty}…{epsf.sty}`; psfig.sty and
  // epsfsafe.tex are not in TeX Live, so epsfig's `\psfig` serves), so `\psfig`, `\epsfig` and `\epsfbox` need no
  // package of the paper's own (run 340 ~650 papers; witnesses 0709.0836, 1310.3924, 0709.3996). A class version that
  // inputs none of them (ws-mpla.cls of 2508.15289) gets them too: one binding serves the family's union.
  RequirePackage!("epsfig");
  RequirePackage!("epsf");
  // ws-mpla.cls:277-282 (ws-ijmpa.cls:288 the same) `\pub{received}{revised}`: the publication history under the
  // author block, the template's "Received (Day Month Year)" kept as the paper gives it (run 340 ~500 papers;
  // witnesses 2508.15289, gr-qc0206056, astro-ph0505113). An empty argument (`\pub{}{}`) makes no note.
  DefMacro!(
    "\\pub{}{}",
    "\\if\\relax\\detokenize{#1}\\relax\\else\\lx@add@pubnote[role=received]{#1}\\fi\
     \\if\\relax\\detokenize{#2}\\relax\\else\\lx@add@pubnote[role=revised]{#2}\\fi"
  );
  // ws-ijmpa.cls:152 (ws-ijmpe.cls:1184, ws-procs9x6.cls:348) `\cdash`: a raised en dash, as in "C\cdash H".
  DefMacro!("\\cdash", "\\ensuremath{^{\\hbox{--}}}");
  // ws-mpla.cls:451-482 the class's lists: `{alphlist}`, `{arabiclist}` and `{romanlist}` are numbered (a), (1), (i)
  // lists with their second level, `{itemlist}` a bulleted one (its enumerate takes `\labelitemi` for every label).
  RawTeX!(
    r"\newenvironment{alphlist}{\def\theenumi{\alph{enumi}}\def\theenumii{\roman{enumii}}\def\labelenumi{(\theenumi)}\def\labelenumii{(\theenumii)}\begin{enumerate}}{\end{enumerate}}
\newenvironment{romanlist}{\def\theenumi{\roman{enumi}}\def\theenumii{\alph{enumii}}\def\labelenumi{(\theenumi)}\def\labelenumii{(\theenumii)}\begin{enumerate}}{\end{enumerate}}
\@ifundefined{arabiclist}{\newenvironment{arabiclist}{\def\theenumi{\arabic{enumi}}\def\theenumii{\alph{enumii}}\def\labelenumi{(\theenumi)}\def\labelenumii{(\theenumii)}\begin{enumerate}}{\end{enumerate}}}{}
\newenvironment{itemlist}{\begin{itemize}}{\end{itemize}}"
  );

  // \catchline{vol}{issue}{year}{pageStart}{pageEnd} — bibliographic
  // running-header metadata. Preserve as frontmatter note.
  DefMacro!(
    "\\catchline{}{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=catchline]{Vol. #1, No. #2 (#3) #4--#5}"
  );

  // \Journal{...} {...}{...}{...} — citation format helper (4-arg per WS conv).
  // Render the args inline as a generic citation string.
  DefMacro!("\\Journal{}{}{}{}", "{#1} {\\bf #2}, #3 (#4)");

  // No-op layout helpers.
  def_macro_noop("\\copyrightheading{}")?;
  def_macro_noop("\\paperBack")?;
  DefMacro!("\\catchlinefont", "\\footnotesize");

  // \ccode{...} — WS classification-codes block (PACS-style codes in
  // ws-ijmpe and friends). Real def is styled paragraph; preserve as
  // a classification note. Witness 2307.12748, 2307.16467.
  DefMacro!(
    "\\ccode{}",
    "\\@add@frontmatter{ltx:classification}[scheme=PACS]{#1}"
  );
  // \category{code}{name} — ws classification 2-arg variant.
  DefMacro!(
    "\\category{}{}",
    "\\@add@frontmatter{ltx:classification}[scheme=#1]{#2}"
  );

  // {history} env — publication-history wrapper (received/revised/
  // accepted dates). Preserve as a frontmatter note. Witness
  // 2307.12748, 2307.16467 + 2 stage-3/4 ws papers.
  DefEnvironment!("{history}",
    "<ltx:note role='history'>#body</ltx:note>",
    mode => "internal_vertical");
  // \received{date}, \revised{date}, \accepted{date}, \comby{name}
  // — used inside {history}. Preserve as inline notes.
  DefMacro!("\\received{}", "Received #1\\par");
  DefMacro!("\\revised{}", "Revised #1\\par");
  DefMacro!("\\accepted{}", "Accepted #1\\par");
  // ws-*.cls: \def\published#1{Published~#1\par}. Witness 2411.01389 (ws-ijmpa).
  DefMacro!("\\published{}", "Published~#1\\par");
  DefMacro!("\\comby{}", "Communicated by #1\\par");
  // \email / \http / \uurl — render contact info inline.
  DefMacro!("\\email{}", "\\textit{#1}\\par");
  DefMacro!("\\http{}", "\\textit{http://#1}\\par");
  DefMacro!("\\uurl{}", "\\textit{#1}\\par");

  // ws-mpla.cls and other ws classes: \tbl{caption}{body} — table
  // caption + tabular body wrapper. Real def is essentially
  // \caption{#1}<body #2 in tabular>. Bridge to \caption{caption}
  // followed by the body content. Witness 2404.18954 (ws-mpla).
  DefMacro!("\\tbl{}{}", "\\caption{#1}#2");
  // \ttbl{label}{caption}{body} — 3-arg variant with explicit label.
  DefMacro!("\\ttbl{}{}{}", "\\caption{#2}\\label{#1}#3");
  // \tcaption{caption} — single-arg variant in some ws families.
  DefMacro!("\\tcaption{}", "\\caption{#1}");
  // ws-*.cls L406 `\def\refcite{\@ifnextchar[...\@refcitex...}` — a bare-number
  // citation variant (used as "Ref.~\refcite{key}" → "Ref. 5"). The ws_journal
  // binding lacked it → undefined (Perl defines it). Map to \cite (a linked
  // citation; the bare-vs-bracketed number is a post-processing nuance). Accepts
  // comma-separated keys like \cite. Witness 2306.15982 (ws-ijmpd).
  DefMacro!("\\refcite{}", "\\cite{#1}");

  // ws-procs9x6.cls:721-731, ws-ijmpa.cls:685-694 (every ws class, the lengths differing; ws-ijmpb.cls:653-658 in
  // cond-mat0509290's copy; ws-rv9x6.cls:618-620 an `\hline` for `\Hline`): the table rules. `\Hline` is a
  // double-weight `\hline`; the others add a rule (or `\crule`'s argument, a `\cline`) between rows that are empty and
  // of negative height, which the alignment drops, so each is the rule it draws. Witness 0704.0076 (ws-ijmpa).
  RawTeX!(
    r"\def\Hline{\noalign{\ifnum0=`}\fi\hrule \@height 2\arrayrulewidth \futurelet\@tempa\@xhline}
\def\toprule{\Hline\\[-6.5pt]}
\def\colrule{\\[-7pt]\hline\\[-6pt]}
\def\TMcolrule{\\[-9.5pt]\hline\\[-9.5pt]}
\def\botrule{\\[-7pt]\Hline}
\def\crule#1{\\[-7pt]#1\\[-6pt]}"
  );
});
