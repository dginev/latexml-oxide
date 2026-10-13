//! Stub for MDPI journal class (Definitions/mdpi.cls, bundled by users).
//!
//! Real mdpi.cls L20-50 loads article + many packages including hyperref,
//! url, booktabs, ragged2e (for \justify), cleveref. Mirror those so
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
  DeclareOption!("submit", "\\gdef\\@status{submit}");
  DeclareOption!("accept", "\\gdef\\@status{accept}");
  DeclareOption!(None, {});
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
  RequirePackage!("cleveref");
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
  let shipped = crate::shipped_class::shipped_class_text("mdpi")?;
  for package in [
    "caption", "lastpage", "float", "enumitem", "mathpazo", "setspace", "epstopdf", "upgreek",
    "subfig",
  ] {
    if shipped
      .as_deref()
      .is_none_or(|text| crate::shipped_class::class_text_loads(text, package))
    {
      RequirePackage!(package);
    }
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
  DefMacro!(
    "\\pubvolume{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}"
  );
  DefMacro!("\\pubyear{}", "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!(
    "\\issuenum{}",
    "\\@add@frontmatter{ltx:note}[role=issue]{#1}"
  );
  DefMacro!(
    "\\reftitle{}",
    "\\@add@frontmatter{ltx:note}[role=reftitle]{#1}"
  );
  def_macro_noop("\\PublishersNote")?;
  DefMacro!(
    "\\articlenumber{}",
    "\\@add@frontmatter{ltx:note}[role=articlenumber]{#1}"
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
  def_macro_noop("\\orcidA")?;
  def_macro_noop("\\orcidB")?;
  def_macro_noop("\\orcidC")?;
  def_macro_noop("\\orcidD")?;
  def_macro_noop("\\orcidE")?;
  def_macro_noop("\\orcidF")?;
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
    DefMacro!(
      "\\doinum{}",
      "\\ifx\\@status\\lx@mdpi@submit\\else\\@add@frontmatter{ltx:note}[role=doi]{#1}\\fi"
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
  }
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
