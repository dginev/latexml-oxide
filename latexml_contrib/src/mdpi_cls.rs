//! Stub for MDPI journal class (Definitions/mdpi.cls, bundled by users).
//!
//! Real mdpi.cls L20-50 loads article + many packages including hyperref,
//! url, booktabs, ragged2e (for \justify), cleveref (the copies since 2023). Mirror those so
//! papers using \href, \hypersetup, \url, \justify, \crefrangelabelformat
//! don't error out. Witness 2410.21443.
use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};
use latexml_package::prelude::*;

LoadDefinitions!({
  // mdpi.cls:109-110, 120: the `submit` (the default) and `accept` statuses; a submitted manuscript's title page
  // prints "Version … submitted to …" where an accepted one prints its `\history` (mdpi.cls:664-670). The journal and
  // type options are the printed page's, and the rest, which mdpi.cls:117 passes to article (size, paper), set only
  // its layout, so none is passed on.
  RawTeX!(r"\def\@status{submit}\def\lx@mdpi@submit{submit}");
  // The journal options: the copies since 2015 read them from the journal table the paper ships beside the class
  // (2208.12066's mdpi.cls :74 `\input{Definitions/journalnames}`, 41 copies `\input{journalnames}`), a row per
  // journal (`\DeclareOption{sensors}{\gdef\@journal{sensors}…\gdef\@doiabbr{s}…}`), and :139 executes the class's
  // default (`notspecified`, `entropy` in older copies) before the paper's: the journal's DOI abbreviation, which the
  // class's default DOI is made of (`default_doi`, read by `\linksupplementary`).
  let shipped = crate::shipped_class::shipped_class_text("mdpi")?;
  let journal_table = shipped.as_deref().and_then(journal_table_input);
  if let Some(table) = &journal_table {
    RawTeX!(&s!("\\InputIfFileExists{{{table}}}{{}}{{}}"));
  }
  DeclareOption!("submit", "\\gdef\\@status{submit}");
  DeclareOption!("accept", "\\gdef\\@status{accept}");
  DeclareOption!(None, {});
  if journal_table.is_some()
    && let Some(defaults) = shipped
      .as_deref()
      .and_then(|text| crate::shipped_class::class_code_argument(text, "\\ExecuteOptions"))
  {
    RawTeX!(&s!("\\ExecuteOptions{{{defaults}}}"));
  }
  ProcessOptions!();
  LoadClass!("article");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("graphicx");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("url");
  RequirePackage!("booktabs");
  RequirePackage!("ragged2e");
  // cleveref only where the class loads it (2410.21443's 2025 copy :445 `\RequirePackage[capitalise,noabbrev]{cleveref}`,
  // 2401.15064's, 2408.03136's), or with no copy shipped: the 2021-2022 copies load none (2111.02495, 2201.05363,
  // 2212.11355), and cleveref's caption hooks under their newfloat and caption loads (mdpi.cls:1184-1192) and the
  // paper's subcaption made every `\caption` in a `{subfigure}` err (15 and 40 Errors; pdflatex 0).
  if shipped
    .as_deref()
    .is_none_or(|text| crate::shipped_class::class_text_loads(text, "cleveref"))
  {
    RequirePackage!("cleveref");
  }
  RequirePackage!("etoolbox");
  RequirePackage!("lineno");
  // MDPI papers use {adjustwidth} from changepage. Witness 2503.13839 +5.
  RequirePackage!("changepage");
  // mdpi.cls also requires these for table + citation + endnote markup
  // (L50 multirow, L58 tabularx, L240 natbib sort&compress, L65 enotez).
  // Without them papers using \citep/\citet, \multirow, {tabularx},
  // \endnote/\printendnotes hit undefined-CS cascades. Witness
  // 2003.10420 (CONVERR_9: \citep/\citet/\multirow/{tabularx} +
  // \endnote/\printendnotes + mdpi-specific \tablesize/\fulllength).
  RequirePackage!("natbib");
  // mdpi.cls:48 loads soul for its highlighting (`\hl`, 2312.16815; Perl's scan of the class loads it).
  RequirePackage!("soul");
  // mdpi.cls:381-385: cross-reference shorthands.
  RawTeX!(
    r"\providecommand{\fig}[1]{Figure~\ref{#1}}
\providecommand{\tabref}[1]{Table~\ref{#1}}
\providecommand{\sect}[1]{Section~\ref{#1}}
\providecommand{\app}[1]{Appendix~\ref{#1}}
\providecommand{\sche}[1]{Scheme~\ref{#1}}"
  );
  RequirePackage!("multirow");
  RequirePackage!("tabularx");
  RequirePackage!("makecell");
  RequirePackage!("array");
  RequirePackage!("colortbl");
  // mdpi.cls:34 `\RequirePackage{fancyhdr}` and :1537-1559 geometry (per page
  // layout): papers restyle the page with `\fancyhead`/`\fancypagestyle` and
  // `\newgeometry` — undefined floods into `Fatal:TooManyErrors` (arXiv
  // 2605.22427, 2605.29102). Layout only; the bindings accept them.
  RequirePackage!("fancyhdr");
  RequirePackage!("geometry");
  // The packages the mdpi.cls versions load that papers call into (of 119 sampled copies, 2012-2025): caption and
  // lastpage (all 119; `\ContinuedFloat` 25, `\captionsetup` 22, `\captionof` 16 run-340 first errors), float (119),
  // enumitem (105; `\setitemize` 72), mathpazo (105), setspace (105; `\setstretch`), epstopdf (101; `\epstopdfsetup`
  // 7), upgreek (49; `\upmu` 97) and subfig (14; `\subfloat` 30). Witnesses 1904.12911, 2110.08429, 1904.02732,
  // 2004.08650, 2508.02963, 2508.18441. The paper's own copy decides which it loads (a 2013 copy loads no enumitem, so a
  // paper's `\usepackage[shortlabels]{enumitem}` is the first load); without one, all of them.
  for package in [
    "caption", "lastpage", "float", "enumitem", "mathpazo", "setspace", "epstopdf", "upgreek",
    "subfig", "paracol",
  ] {
    if shipped
      .as_deref()
      .is_none_or(|text| crate::shipped_class::class_text_loads(text, package))
    {
      RequirePackage!(package);
    }
  }
  // mdpi.cls:1362-1373 (2302.03012's copy; the same settings in the copies of 22 mdpi papers of the sel78d 30k census,
  // some with `margin=0.75cm` or `singlelinecheck=on`): after caption, the class sets its float types' captions — a bold
  // small label ended by a period, small text ("Figure 1. "). A shipped copy is read after this binding and sets them
  // itself (those 22 papers printed "Figure 1. " before too); without one the binding stands in for the class, which
  // had left caption's default "Figure 1: " (quality ledger 132). The figure and table rows as the class writes them; its
  // newfloat types (`listing`, `boxenv`, `chart`, `scheme`, mdpi.cls:1355-1360) are not defined here. Guard
  // `perfect_kernel_batch78d::mdpi_captions_end_the_label_with_a_period`.
  if shipped.is_none() {
    RawTeX!(
      r"\captionsetup[figure]{position=bottom, labelfont={bf, small, stretch=1.17}, labelsep=period, textfont={small, stretch=1.17}, aboveskip=6pt, belowskip=-6pt, singlelinecheck=off, justification=justified}
\captionsetup[table]{position=top, labelfont={bf, small, stretch=1.17}, labelsep=period, textfont={small, stretch=1.17}, aboveskip=6pt, singlelinecheck=off, justification=justified}"
    );
  }
  // The copies that load paracol (2105.08579's, 2201.05363's, 1809.00601's: :55 `\RequirePackage{paracol}`) open
  // `\begin{paracol}{2}` for the first page's side column (citation, editor, dates) and `\switchcolumn` to the body,
  // in `\AfterEndPreamble` after the class's own `\maketitle` (2105.08579's :880-887, directly; 1809.00601's :908-912,
  // through `\maketitlen`), and the template's paper closes it with `\end{paracol}` before its references: without
  // the opening, that `\end{paracol}` closed the wrong group (run 340: the `\endgroup` error of ~470 mdpi papers). The
  // side column's content is the frontmatter's here, so the body starts at the switch. Only a copy that opens paracol
  // does so (1307.3961's 2013 copy does not); with no copy shipped, nothing is opened. Guards
  // `perfect_kernel_batch66b::{mdpi_opens_the_paracol_the_paper_closes, mdpi_without_paracol_opens_nothing}`.
  if shipped.as_deref().is_some_and(|text| {
    crate::shipped_class::class_text_loads(text, "paracol")
      && crate::shipped_class::class_code_contains(text, "\\begin{paracol}")
  }) {
    RawTeX!(r"\AfterEndPreamble{\begin{paracol}{2}\switchcolumn}");
  }
  // mdpi.cls:178-262: amsthm with the `mdpi` style and the capitalised
  // environments on their own counters (`\begin{Proposition}`, `{Remark}`,
  // `{Lemma}` …). `{Algorithm}` is left out: its `algorithm` counter would
  // collide with a paper's own `\usepackage{algorithm}` float.
  RequirePackage!("amsthm");
  RawTeX!(
    r"\newtheoremstyle{mdpi}{12pt}{12pt}{\itshape}{}{\bfseries}{.}{.5em}{}
\theoremstyle{mdpi}
\newcounter{theorem}\newtheorem{Theorem}[theorem]{Theorem}
\newcounter{lemma}\newtheorem{Lemma}[lemma]{Lemma}
\newcounter{corollary}\newtheorem{Corollary}[corollary]{Corollary}
\newcounter{proposition}\newtheorem{Proposition}[proposition]{Proposition}
\newcounter{characterization}\newtheorem{Characterization}[characterization]{Characterization}
\newcounter{property}\newtheorem{Property}[property]{Property}
\newcounter{problem}\newtheorem{Problem}[problem]{Problem}
\newcounter{example}\newtheorem{Example}[example]{Example}
\newcounter{examplesanddefinitions}\newtheorem{ExamplesandDefinitions}[examplesanddefinitions]{Examples and Definitions}
\newcounter{remark}\newtheorem{Remark}[remark]{Remark}
\newcounter{definition}\newtheorem{Definition}[definition]{Definition}
\newcounter{hypothesis}\newtheorem{Hypothesis}[hypothesis]{Hypothesis}
\newcounter{notation}\newtheorem{Notation}[notation]{Notation}
\newcounter{assumption}\newtheorem{Assumption}[assumption]{Assumption}"
  );
  // mdpi.cls:1655-1657: the template's `tabularx` column types. Without them
  // `\begin{tabularx}{\textwidth}{CCC}` has one column and every `&` is an
  // "Extra alignment tab" (arXiv 2605.17530 ×501, 2605.22427, 2605.29102).
  RawTeX!(
    r"\newcolumntype{C}{>{\centering\arraybackslash}X}
\newcolumntype{L}{>{\raggedright\arraybackslash}X}
\newcolumntype{R}{>{\raggedleft\arraybackslash}X}"
  );

  // mdpi.cls L1297-1298: `\newlength{\fulllength}` (page-rule width).
  // Pure layout — define as a register so `\rule{\fulllength}{..}` and
  // friends resolve. Value irrelevant to XML output.
  DefRegister!("\\fulllength" => Dimension::new(0));
  // mdpi.cls L1114-1115: `\def\@tablesize{}` +
  // `\newcommand{\tablesize}[1]{\gdef\@tablesize{#1}}`. Sets the font
  // size used inside `tabularx` — typesetting-only; store the arg in
  // `\@tablesize` (some code probes `\ifx\@tablesize\@empty`).
  DefMacro!("\\@tablesize", "");
  DefMacro!("\\tablesize{}", "\\gdef\\@tablesize{#1}");
  // enotez/endnotes endnote API. mdpi.cls L65 loads enotez; we have no
  // enotez binding, so define the two endnote commands authors use.
  // `\endnote{text}` → render inline as an ltx:note (footnote-style);
  // `\printendnotes` is the deferred-output hook — no-op (the notes are
  // already emitted inline). Mirrors how Perl LaTeXML's endnotes.sty
  // handling routes endnotes to ltx:note.
  DefConstructor!("\\endnote{}", "<ltx:note role='endnote'>#1</ltx:note>", sizer => sub[_whatsit] { Ok(out_of_line_size()) });
  def_macro_noop("\\printendnotes")?;

  // MDPI frontmatter — preserve author content as ltx:note frontmatter.
  DefMacro!("\\corresref[]{}", "\\textsuperscript{*#1}");
  def_macro_noop("\\externalbibliography{}")?;
  // \firstpage{N} also defines \@firstpage in the real mdpi.cls;
  // some papers reference it via `\setcounter{page}{\@firstpage}`.
  // Witness 2503.04598 — bytedance_seed paper using the mdpi pattern.
  DefMacro!(
    "\\firstpage{}",
    "\\def\\@firstpage{#1}\\@add@frontmatter{ltx:note}[role=firstpage]{#1}"
  );
  DefMacro!("\\@firstpage", "1");
  DefMacro!(
    "\\firstpagenote{}",
    "\\@add@frontmatter{ltx:note}[role=firstpagenote]{#1}"
  );
  DefMacro!(
    "\\corres[]{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#2}"
  );
  DefMacro!(
    "\\Journal{}",
    "\\@add@frontmatter{ltx:note}[role=journal]{#1}"
  );
  DefMacro!(
    "\\firstnote{}",
    "\\@add@frontmatter{ltx:note}[role=firstnote]{#1}"
  );
  DefMacro!(
    "\\Address{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#1}"
  );
  DefMacro!(
    "\\AuthorNames{}",
    "\\@add@frontmatter{ltx:note}[role=authornames]{#1}"
  );
  DefMacro!(
    "\\AuthorCitation{}",
    "\\@add@frontmatter{ltx:note}[role=authorcitation]{#1}"
  );
  DefMacro!(
    "\\dates{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=dates]{Received: #1 Revised: #2 Accepted: #3}"
  );
  DefMacro!(
    "\\authorinitials{}",
    "\\@add@frontmatter{ltx:note}[role=authorinitials]{#1}"
  );
  // Additional MDPI frontmatter macros (mdpi.cls L530+). Witness 2412.13512.
  DefMacro!("\\Title{}", "\\title{#1}");
  DefMacro!(
    "\\TitleCitation{}",
    "\\@add@frontmatter{ltx:note}[role=titlecitation]{#1}"
  );
  // The volume, issue and article number are also kept as the class keeps them (2208.12066's copy :545, :588, :606-612:
  // `\cutdigits` drops leading zeros; a Molbank article number, `M1234`, as given), for its default DOI
  // (`\linksupplementary`).
  RawTeX!(r"\def\lx@mdpi@molbank{molbank}");
  DefMacro!(
    "\\pubvolume{}",
    "\\gdef\\@pubvolume{\\cutdigits{#1}}\\@add@frontmatter{ltx:note}[role=volume]{#1}"
  );
  DefMacro!("\\pubyear{}", "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!(
    "\\issuenum{}",
    "\\gdef\\@issuenum{\\cutdigits{#1}}\\@add@frontmatter{ltx:note}[role=issue]{#1}"
  );
  DefMacro!(
    "\\reftitle{}",
    "\\@add@frontmatter{ltx:note}[role=reftitle]{#1}"
  );
  def_macro_noop("\\PublishersNote")?;
  DefMacro!(
    "\\articlenumber{}",
    "\\ifx\\@journal\\lx@mdpi@molbank\\gdef\\@articlenumber{#1}\\else\\gdef\\@articlenumber{\\cutdigits{#1}}\\fi\\@add@frontmatter{ltx:note}[role=articlenumber]{#1}"
  );
  DefMacro!(
    "\\copyrightyear{}",
    "\\@add@frontmatter{ltx:note}[role=copyright-year]{#1}"
  );
  DefMacro!(
    "\\histreceived{}",
    "\\@add@frontmatter{ltx:note}[role=received]{#1}"
  );
  DefMacro!(
    "\\histrevised{}",
    "\\@add@frontmatter{ltx:note}[role=revised]{#1}"
  );
  DefMacro!(
    "\\histaccepted{}",
    "\\@add@frontmatter{ltx:note}[role=accepted]{#1}"
  );
  DefMacro!(
    "\\historypublished{}",
    "\\@add@frontmatter{ltx:note}[role=published]{#1}"
  );
  // mdpi.cls:384 `\newcommand{\history}[1]{\gdef\@history{#1}}`: the author's dates, labels and all ("Received: 7
  // October 2020; Accepted: …"), printed under the abstract unless the status is `submit` (mdpi.cls:664-670, 679-685);
  // a later `\history` replaces it (`\gdef`).
  // Witnesses 1003.3225, 1910.14533, 2010.14300, 2109.09102, 2002.09734.
  DefMacro!(
    "\\history{}",
    "\\ifx\\@status\\lx@mdpi@submit\\else\\lx@clear@frontmatter{ltx:note}[role=history]\\@add@frontmatter{ltx:note}[role=history]{#1}\\fi"
  );
  // mdpi.cls:295 `\newcommand{\changeurlcolor}[1]{\hypersetup{urlcolor=#1}}` (the class's own title-page DOI links,
  // and papers calling it). Witnesses as for `\history`.
  DefMacro!("\\changeurlcolor{}", "\\hypersetup{urlcolor=#1}");
  def_macro_noop("\\SetCaptionDefault")?;

  // Newer mdpi.cls L668-685 — additional date/metadata setters.
  // Witness 2503.11347, 2503.13839 (\daterevised, \datereceived,
  // \dateaccepted).
  DefMacro!(
    "\\datereceived{}",
    "\\@add@frontmatter{ltx:note}[role=received]{#1}"
  );
  DefMacro!(
    "\\daterevised{}",
    "\\@add@frontmatter{ltx:note}[role=revised]{#1}"
  );
  DefMacro!(
    "\\dateaccepted{}",
    "\\@add@frontmatter{ltx:note}[role=accepted]{#1}"
  );
  DefMacro!(
    "\\datepublished{}",
    "\\@add@frontmatter{ltx:note}[role=published]{#1}"
  );
  DefMacro!(
    "\\datecorrected{}",
    "\\@add@frontmatter{ltx:note}[role=corrected]{#1}"
  );
  DefMacro!(
    "\\dateretracted{}",
    "\\@add@frontmatter{ltx:note}[role=retracted]{#1}"
  );
  DefMacro!(
    "\\externaleditor{}",
    "\\@add@frontmatter{ltx:note}[role=external-editor]{#1}"
  );
  DefMacro!("\\LSID{}", "\\@add@frontmatter{ltx:note}[role=lsid]{#1}");
  DefMacro!(
    "\\PACS{}",
    "\\@add@frontmatter{ltx:classification}[scheme=PACS]{#1}"
  );
  DefMacro!(
    "\\MSC{}",
    "\\@add@frontmatter{ltx:classification}[scheme=MSC]{#1}"
  );
  DefMacro!(
    "\\JEL{}",
    "\\@add@frontmatter{ltx:classification}[scheme=JEL]{#1}"
  );
  DefMacro!(
    "\\keyword{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}"
  );
  DefMacro!(
    "\\dataset{}",
    "\\@add@frontmatter{ltx:note}[role=dataset]{#1}"
  );
  DefMacro!(
    "\\datasetlicense{}",
    "\\@add@frontmatter{ltx:note}[role=dataset-license]{#1}"
  );

  // Additional newer mdpi.cls macros — preserve content.
  // \Author{name} (capital) is the MDPI variant; route to LaTeX \author.
  DefMacro!("\\Author{}", "\\author{#1}");
  DefMacro!(
    "\\hreflink{}",
    "\\@add@frontmatter{ltx:note}[role=hreflink]{#1}"
  );
  // mdpi.cls (2311.18672's 2023 copy :633-635, as since 2018) `\foreach \x in {A, ..., Z}` defines `\orcidA`…`\orcidZ`,
  // the linked ORCID icon after an author's name, of the iD the paper gives `\orcidauthorA`…
  // (`\newcommand{\orcidauthorA}{0000-…}`): the author's ORCID (2311.18672 calls `\orcidG` and `\orcidH`); without
  // its iD, nothing.
  for letter in 'A'..='Z' {
    RawTeX!(&s!(
      "\\def\\orcid{letter}{{\\@ifundefined{{orcidauthor{letter}}}{{}}{{\\expandafter\\lx@add@orcid\\expandafter{{\\orcidauthor{letter}}}}}}}"
    ));
  }
  // \extralength is a length register — define as 0pt.
  DefRegister!("\\extralength" => Dimension::new(0));
  // \authorcontributions, \funding, \conflictsofinterest,
  // \abbreviations — substantive author-supplied text; render as
  // a named section.
  DefMacro!(
    "\\authorcontributions{}",
    "\\section*{Author Contributions}#1"
  );
  DefMacro!("\\funding{}", "\\section*{Funding}#1");
  DefMacro!(
    "\\conflictsofinterest{}",
    "\\section*{Conflicts of Interest}#1"
  );
  DefMacro!("\\abbreviations{}{}", "\\section*{#1}#2");
  // Additional MDPI back-matter macros (mdpi.cls L1199-1240). These are
  // simple `\newcommand` stubs in the raw cls but in practice the cls
  // raw-load stops or truncates partway through, leaving these undefined.
  // Defining here ensures the back-matter content is preserved.
  DefMacro!("\\supplementary{}", "\\section*{Supplementary Materials}#1");
  DefMacro!(
    "\\institutionalreview{}",
    "\\section*{Institutional Review Board Statement}#1"
  );
  DefMacro!(
    "\\informedconsent{}",
    "\\section*{Informed Consent Statement}#1"
  );
  DefMacro!(
    "\\dataavailability{}",
    "\\section*{Data Availability Statement}#1"
  );
  DefMacro!(
    "\\publicinvolvement{}",
    "\\section*{Public Involvement Statement}#1"
  );
  DefMacro!(
    "\\guidelinesstandards{}",
    "\\section*{Guidelines and Standards Statement}#1"
  );
  DefMacro!(
    "\\entrylink{}",
    "\\@add@frontmatter{ltx:note}[role=entrylink]{#1}"
  );
  DefMacro!(
    "\\reviewreports{}",
    "\\@add@frontmatter{ltx:note}[role=reviewreports]{#1}"
  );
  // \sampleavailability (older mdpi) — sister of \dataavailability.
  DefMacro!(
    "\\sampleavailability{}",
    "\\section*{Sample Availability}#1"
  );
  // \patents — research patents declaration.
  DefMacro!("\\patents{}", "\\section*{Patents}#1");
  // mdpi.cls (2021+) `\publishersnote`: the "Publisher's Note:" back-matter statement (a version's convenience, as the
  // setters below).
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    DefMacro!("\\publishersnote{}", "\\section*{Publisher's Note}#1");
  }
  // \address[id]{text} — preserve as ltx:note.
  DefMacro!(
    "\\address[]{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#2}"
  );
  // \natexlab from natbib — emit arg inline (used in \bibitem to mark
  // companion years like (1999a)/(1999b)).
  DefMacro!("\\natexlab{}", "#1");
  // \textls (microtype letterspacing) — emit as-is.
  DefMacro!("\\textls[]{}", "#2");

  // \isAPAStyle / \isChicagoStyle — branchers in mdpi.cls L450+ that
  // pick references style based on \@journal. The cls defines them as
  // `\newcommand{\isAPAStyle}{\ifthenelse{...}}` and uses them as
  // `\isAPAStyle{...}{...}`. Stub as content-discarder so the
  // bibliography block doesn't emit an undefined error for every paper
  // shipping a local Definitions/mdpi.cls. Witness 2412.13512, 2503.13839.
  DefMacro!("\\isAPAStyle{}{}", "#2");
  DefMacro!("\\isChicagoStyle{}{}", "#2");
  // mdpi.cls:170-175 `\isPreprints`, the same brancher on `\@journal`
  // (preprints / preprintschicago / preprintsapa): a journal paper takes the
  // second branch (`\isPreprints{\centering}{}`, arXiv 2605.17530).
  DefMacro!("\\isPreprints{}{}", "#2");
  // The versions' title-page setters and conveniences (union of 119 sampled copies, 2012-2025; line numbers from
  // 2207.12053's copy). One version defines what another lacks, and a paper whose copy lacks one defines it itself, so
  // they are a fallback's guesses its own definition replaces (`DefinitionOrigin::Fallback`, as jfm_cls.rs). Run 340
  // first errors: `\doinum` 312, `\lastpage` 148, `\secondnote` 145, `\updates` 111, `\orcidicon` 51,
  // `\appendixsections` 42, `\cellWidtha` 31, `\startlandscape` 27, `\conference` 17 (witnesses 2008.03867,
  // 1309.7149, 2103.15665, 2103.03626, 2103.07786, 1904.10935, 2204.11608, 2103.11824, 2008.00705).
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    // mdpi.cls:312 (2025) `\isAPAandChicago`, the same brancher for the journals that cite in both styles.
    DefMacro!("\\isAPAandChicago{}{}", "#2");
    // `\doinum` (:424): the article's DOI, which the journal line prints once the paper is accepted (:446, :646,
    // :667), as `\history` is printed.
    // (and :424 `\gdef\@doinum{#1}`, which `\linksupplementary` reads)
    RawTeX!(r"\def\@doinum{}");
    // :482-505 (and :533-541 `\cutdigits`) the padding the default DOI's issue and article number take, `\fourdigits`
    // in the 2016-2018 copies (54 of 319 sampled: 1607.02330, 1810.09793).
    RawTeX!(
      r"\def\cutdigits#1{\ifnum#1>0 \number#1 \else\ifnum#1<10000 \number#1 \else error\fi\fi}
\def\twodigits#1{\ifnum#1<10 0\number#1 \else\number#1 \fi}
\def\fourdigits#1{\ifnum#1<10 000\number#1 \else\ifnum#1<100 00\number#1 \else\ifnum#1<1000 0\number#1 \else
\ifnum#1<10000 \number#1 \else error\fi\fi\fi\fi}
\def\fivedigits#1{\ifnum#1<10 000\number#1 \else\ifnum#1<100 00\number#1 \else\ifnum#1<1000 0\number#1 \else
\ifnum#1<100000 \number#1 \else error\fi\fi\fi\fi}"
    );
    DefMacro!(
      "\\doinum{}",
      "\\gdef\\@doinum{#1}\\ifx\\@status\\lx@mdpi@submit\\else\\@add@frontmatter{ltx:note}[role=doi]{#1}\\fi"
    );
    // `\secondnote`, `\thirdnote` (:560-563): the author notes after `\firstnote` (:970-980); `\conference` (:521):
    // the note "This paper is an extended version of our paper published in …" (:964-965).
    DefMacro!(
      "\\secondnote{}",
      "\\@add@frontmatter{ltx:note}[role=secondnote]{#1}"
    );
    DefMacro!(
      "\\thirdnote{}",
      "\\@add@frontmatter{ltx:note}[role=thirdnote]{#1}"
    );
    DefMacro!(
      "\\conference{}",
      "\\@add@frontmatter{ltx:note}[role=conference]{This paper is an extended version of our paper published in #1.}"
    );
    // `\featuredapplication`, `\simplesumm` (:552-575): the "Featured Application" and "Simple Summary" blocks
    // printed before the abstract (`\abstractkeywords`, :1019-1034). Notes, not abstracts: a paper's `\abstract`
    // replaces every frontmatter abstract (`\lx@add@abstract`).
    DefMacro!(
      "\\featuredapplication{}",
      "\\@add@frontmatter{ltx:note}[role=featuredapplication]{#1}"
    );
    DefMacro!(
      "\\simplesumm{}",
      "\\@add@frontmatter{ltx:note}[role=simplesummary]{#1}"
    );
    // `\addhighlights{yes}` (:592-593) only switches the "Highlights" block on: the class prints `\addhighlights`
    // again (:1035-1039), which the paper has redefined to hold the highlights ("redefined in .tex to use itemize").
    // So the block is the paper's own `\addhighlights`, when the switch is set and the paper replaced the setter.
    RawTeX!(
      r"\def\@addhighlights{}
\newcommand\lx@mdpi@addhighlights[1]{\gdef\@addhighlights{#1}}
\let\addhighlights\lx@mdpi@addhighlights
\AtBeginDocument{\ifx\@addhighlights\@empty\else\ifx\addhighlights\lx@mdpi@addhighlights\else
  \@add@frontmatter{ltx:note}[role=highlights]{\addhighlights}\fi\fi}"
    );
    def_macro_noop("\\abstractkeywords")?;
    // The 2012-2015 versions' `\acknowledgements`/`\conflictofinterests` set their argument, the section's title, as a
    // bold paragraph (1307.3961's copy :1466-1473); later versions keep `\conflictofinterests` for compatibility with
    // the statement as its argument under a bold "Conflicts of Interest:" label (43 of the 62 sampled copies that
    // define it; one sets the argument alone). The paper's own copy decides; without one, the label form.
    DefMacro!("\\acknowledgements{}", "\\par\\noindent\\textbf{#1}\\par");
    let conflicts = match shipped.as_deref().map(conflict_of_interests_form) {
      Some(ConflictForm::Title) => "\\par\\noindent\\textbf{#1}\\par",
      Some(ConflictForm::Plain) => "\\par\\noindent #1\\par",
      _ => "\\par\\noindent\\textbf{Conflicts of Interest:} #1\\par",
    };
    RawTeX!(&s!("\\def\\conflictofinterests#1{{{conflicts}}}"));
    // Page and layout settings: `\lastpage`, `\continuouspages` (page ranges of the journal line), `\updates` (the
    // check-for-updates logo link, :794), `\longauthorlist`, `\appendixsections`, the ORCID logo `\orcidicon` (an
    // image file the papers' Definitions/ folder holds), `\startlandscape`/`\finishlandscape`, `\widefigure` and
    // `\widetable` (page geometry and caption margins).
    for setter in [
      "\\lastpage{}",
      "\\continuouspages{}",
      "\\updates{}",
      "\\longauthorlist{}",
      "\\appendixsections{}",
    ] {
      def_macro_noop(setter)?;
    }
    for switch in [
      "\\orcidicon",
      "\\startlandscape",
      "\\finishlandscape",
      "\\widefigure",
      "\\widetable",
    ] {
      def_macro_noop(switch)?;
    }
    // `\highlighting` (a yellow `\colorbox`), the table column widths `\cellWidtha`…`\cellWidthz` and
    // `\PreserveBackslash` (the XML2PDF column helper).
    DefMacro!("\\highlighting{}", "\\colorbox{yellow}{#1}");
    for letter in 'a'..='z' {
      RawTeX!(&s!("\\newlength{{\\cellWidth{letter}}}"));
    }
    RawTeX!(r"\newcommand{\PreserveBackslash}[1]{\let\temp=\\#1\let\\=\temp}");
    // More of the versions' commands papers call (run 347: 45 mdpi and 31 Definitions/mdpi papers; line numbers from
    // 2404.14200's 2023 copy unless named):
    // - `\conferencetitle{name}` (1708.02914's 2017 copy :781; 23 copies, 18 papers: 1709.06513, 1609.04583), the
    //   conference a proceedings paper was given at, printed in the footer: the conference note;
    // - `\contributed{text}` (2015, 1506.06852's copy :627-628; 7 papers: 1507.03569, 1510.06668), the "These authors
    //   contributed equally" note; `\fourthnote`, `\fifthnote` (:611-614; 1902.06114, 2005.05017), the author notes
    //   after `\thirdnote`; `\emails{…}` (2008, 0912.1688's copy :127, :334 "E-mail: …"), the authors' e-mail;
    // - `\orcidauthorONE{id}`…`\orcidauthorSIX` and `\orcidONE`…`\orcidSIX` (2017, 1708.02914's copy :798-817; 1707.09764,
    //   1709.06453, 1710.03078), an author's ORCID set in the preamble and its linked icon after the name: the author's
    //   ORCID;
    // - `\conflictofinterest{text}` (1911.06360's copy :862, the singular "for book production"; 1710.04728, 1811.06122),
    //   `\useofartificialintelligence{text}` (:1248; 2609.34305), the back matter's labelled statements;
    // - `\linksupplementary{file}` (:1498; 214 copies, 19 papers: 2208.12066, 2311.18672), the link to the article's
    //   supplementary file by its `\doinum`; `\bio{photo}{text}` (:1262; 2107.14152, 2111.02495), an author's
    //   biography beside the photo; `\justifyorcenter{text}` (:1461), a table footnote centred when short (2208.12066);
    // - `\highlight{text}` (2017, 1708.02914's :51; 2111.05917), yellow-boxed as `\highlighting`; `\CorrStatement{yes}`
    //   (:641-642), the switch of the correspondence statement (2409.02954); `\dist` (:828), the notes' hanging indent.
    DefMacro!(
      "\\conferencetitle{}",
      "\\@add@frontmatter{ltx:note}[role=conference]{#1}"
    );
    DefMacro!(
      "\\contributed{}",
      "\\@add@frontmatter{ltx:note}[role=contributed]{#1}"
    );
    DefMacro!(
      "\\fourthnote{}",
      "\\@add@frontmatter{ltx:note}[role=fourthnote]{#1}"
    );
    DefMacro!(
      "\\fifthnote{}",
      "\\@add@frontmatter{ltx:note}[role=fifthnote]{#1}"
    );
    DefMacro!("\\emails{}", "\\lx@add@email{#1}");
    for n in ["ONE", "TWO", "THREE", "FOUR", "FIVE", "SIX"] {
      RawTeX!(&s!(
        "\\def\\@orcidauthor{n}{{}}\\newcommand{{\\orcidauthor{n}}}[1]{{\\gdef\\@orcidauthor{n}{{#1}}}}\\newcommand{{\\orcid{n}}}{{\\expandafter\\lx@add@orcid\\expandafter{{\\@orcidauthor{n}}}}}"
      ));
    }
    RawTeX!(
      r"\newcommand{\conflictofinterest}[1]{\par\noindent\textbf{Conflicts of Interest:} #1\par}
\newcommand{\useofartificialintelligence}[1]{\par\noindent\textbf{Use of Artificial Intelligence:} #1\par}
\newcommand{\bio}[2]{\par\noindent{#1}\ {#2}\par}
\newcommand{\justifyorcenter}[1]{#1}
\newcommand{\highlight}[1]{\colorbox{yellow}{#1}}
\def\@CorrStatement{}\newcommand{\CorrStatement}[1]{\gdef\@CorrStatement{#1}}
\newcommand{\dist}{1.7em}"
    );
    // `\linksupplementary{file}` (2404.14200's copy :1498, 2208.12066's :1479: `\url{https://www.mdpi.com/article/
    // \@doinum/#1}`; 19 papers, none setting `\doinum`: 2208.12066, 2602.00080, 1901.06933, 2311.18672, 2607.17752): the
    // link under the paper's DOI, else under the default the copy computes at load (`default_doi`, 2208.12066's :523-530
    // `\doinum{10.3390/\@doiabbr\@pubvolume\twodigits\@issuenum\fivedigits\@articlenumber}`, `10.3390/s23001234`), which
    // needs the copy's journal table and the paper's volume, issue and article number. Without them the DOI is unknown:
    // the file's name, and a Warning (pdflatex has the copy, so its DOI is always known).
    if let Some((article_only, formula)) = shipped.as_deref().and_then(default_doi) {
      RawTeX!(&s!(
        "\\def\\lx@mdpi@doi@default{{{formula}}}\\def\\lx@mdpi@doi@articleonly{{,{},}}",
        article_only.join(",")
      ));
    }
    DefMacro!("\\linksupplementary{}", sub[(file)] {
      let file: Tokens = file;
      let doi = if macro_text("\\@doinum").is_some_and(|doi| !doi.trim().is_empty()) {
        Some(Tokens!(T_CS!("\\@doinum")))
      } else {
        default_doi_tokens()
      };
      Ok(match doi {
        Some(doi) => {
          let mut out = TokenizeInternal!(r"\edef\lx@mdpi@link{\noexpand\url{https://www.mdpi.com/article/").unlist();
          out.extend(doi.unlist());
          out.extend(TokenizeInternal!(r"/\unexpanded{").unlist());
          out.extend(file.unlist());
          out.extend([T_END!(), T_END!(), T_END!(), T_CS!("\\lx@mdpi@link")]);
          Tokens::new(out)
        },
        None => {
          Warn!(
            "missing",
            "\\linksupplementary",
            s!(
              "The article's DOI is unknown (no \\doinum, nor the class copy's journal table and the paper's volume, issue \
               and article number to compute it), so the supplementary file '{}' is not linked",
              file.to_string()
            )
          );
          file
        },
      })
    });
  }
  // The astronomical journal abbreviations some copies paste in (2211.06541, 2212.11355, 2406.01458; 2510.22959's
  // mdpi_mod.cls).
  crate::shipped_class::provide_shipped_aas_macros("mdpi")?;
  // \acknowledgments — newer-mdpi spelling (vs \acknowledgements).
  // Render as structural ltx:acknowledgements so post-processors map it
  // to the canonical role/styling (vs flattening to a generic section).
  DefConstructor!(
    "\\acknowledgments{}",
    "<ltx:acknowledgements>#1</ltx:acknowledgements>"
  );
  // \appendixtitles{Yes|No} / \appendixstart — appendix-numbering
  // toggle. No-op (we don't replay mdpi's appendix counter dance).
  def_macro_noop("\\appendixtitles{}")?;
  def_macro_noop("\\appendixstart")?;
});

/// The journal table the copy `class` (of mdpi.cls) inputs (2208.12066's :74 `\input{Definitions/journalnames}`), if any.
fn journal_table_input(class: &str) -> Option<String> {
  let code = crate::shipped_class::class_code(class);
  code.match_indices("\\input{").find_map(|(at, call)| {
    let rest = &code[at + call.len()..];
    let name = &rest[..rest.find('}')?];
    name
      .contains("journalnames")
      .then(|| name.trim().to_string())
  })
}

/// The DOI the copy `class` (of mdpi.cls) gives an article by default: the journals whose DOI is the article number
/// alone (2208.12066's :515-528 `\ifthenelse{\equal{\@journal}{molbank} \OR …}{\doinum{10.3390/\@articlenumber}}`), and
/// every other journal's, the last default `\doinum` (:529), when it is made of the journal's DOI abbreviation, the
/// volume, issue and article number and the class's padding only (6 copies pad with siunitx's `\num`, which no `\edef`
/// expands: none).
fn default_doi(class: &str) -> Option<(Vec<String>, String)> {
  const ARTICLE_ONLY: &str = "10.3390/\\@articlenumber";
  const PARTS: [&str; 7] = [
    "@doiabbr",
    "@pubvolume",
    "@issuenum",
    "@articlenumber",
    "twodigits",
    "fourdigits",
    "fivedigits",
  ];
  let code = crate::shipped_class::class_code(class);
  let defaults: Vec<(usize, String)> = code
    .match_indices("\\doinum{10.3390/")
    .filter_map(|(at, _)| {
      crate::shipped_class::class_code_argument(&code[at..], "\\doinum")
        .map(|arg| (at, arg.trim().to_string()))
    })
    .collect();
  let (_, formula) = defaults.last()?.clone();
  if formula.contains(['#', '%']) {
    return None;
  }
  let mut rest = formula.as_str();
  while let Some(at) = rest.find(['\\', '[']) {
    let after = rest[at..].strip_prefix('\\')?;
    let len = after
      .find(|c: char| !c.is_ascii_alphabetic() && c != '@')
      .unwrap_or(after.len());
    if !PARTS.contains(&&after[..len]) {
      return None;
    }
    rest = &after[len..];
  }
  let article_only = match defaults.as_slice() {
    [(at, first), .., _] if first == ARTICLE_ONLY => {
      let condition = &code[code[..*at].rfind("\\ifthenelse")?..*at];
      condition
        .match_indices("\\equal{\\@journal}{")
        .filter_map(|(i, call)| {
          let name = &condition[i + call.len()..];
          Some(name[..name.find('}')?].trim().to_string())
        })
        .collect()
    },
    _ => Vec::new(),
  };
  Some((article_only, formula))
}

/// The text of the macro `cs`'s expansion (empty for an empty one, `\gdef\@doiabbr{}`: the `preprints` row), if `cs`
/// is defined.
fn macro_text(cs: &str) -> Option<String> {
  let defn = lookup_definition(&T_CS!(cs)).ok()??;
  Some(match defn.get_expansion() {
    Some(ExpansionBody::Tokens(body)) => body.to_string(),
    _ => String::new(),
  })
}

/// The class's default DOI (`default_doi`) for the paper's journal, volume, issue and article number, when the journal
/// table named the journal and each number is one (`\cutdigits` of digits; a Molbank article number as it is): what
/// the class's `\@doinum` expands to.
fn default_doi_tokens() -> Option<Tokens> {
  let formula = macro_text("\\lx@mdpi@doi@default")?;
  let article_only = macro_text("\\@journal").is_some_and(|journal| {
    macro_text("\\lx@mdpi@doi@articleonly")
      .is_some_and(|list| list.contains(&s!(",{},", journal.trim())))
  });
  let number = |cs: &str| {
    macro_text(cs).is_some_and(|text| {
      let digits: String = text
        .replace("\\cutdigits", "")
        .chars()
        .filter(|c| !matches!(c, '{' | '}') && !c.is_whitespace())
        .collect();
      !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
    })
  };
  if article_only {
    return macro_text("\\@articlenumber")
      .is_some_and(|text| !text.trim().is_empty())
      .then(|| TokenizeInternal!("10.3390/\\@articlenumber"));
  }
  let complete = (!formula.contains("\\@doiabbr") || macro_text("\\@doiabbr").is_some())
    && ["\\@pubvolume", "\\@issuenum", "\\@articlenumber"]
      .iter()
      .all(|cs| !formula.contains(cs) || number(cs));
  complete.then(|| Tokens!(T_CS!("\\lx@mdpi@doi@default")))
}

/// How the paper's copy of mdpi.cls sets `\conflictofinterests`'s argument.
enum ConflictForm {
  /// The 2012-2015 versions: the argument is the section's title, set bold (`\textbf{#1}`).
  Title,
  /// The later versions: the statement under a bold "Conflicts of Interest:" label.
  Label,
  /// One version: the argument alone.
  Plain,
}

/// The form of the `\conflictofinterests` definition in `class` (a copy of mdpi.cls); the label form when it has none.
fn conflict_of_interests_form(class: &str) -> ConflictForm {
  let Some((_, rest)) = class.split_once("\\newcommand{\\conflictofinterests}") else {
    return ConflictForm::Label;
  };
  let body: String = rest.chars().take(300).collect();
  if body.contains("Conflicts of Interest:") {
    ConflictForm::Label
  } else if body.contains("\\textbf{#1}") {
    ConflictForm::Title
  } else {
    ConflictForm::Plain
  }
}
