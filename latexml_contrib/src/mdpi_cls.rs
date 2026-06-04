//! Stub for MDPI journal class (Definitions/mdpi.cls, bundled by users).
//!
//! Real mdpi.cls L20-50 loads article + many packages including hyperref,
//! url, booktabs, ragged2e (for \justify), cleveref. Mirror those so
//! papers using \href, \hypersetup, \url, \justify, \crefrangelabelformat
//! don't error out. Witness 2410.21443.
use latexml_package::prelude::*;


LoadDefinitions!({
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
  RequirePackage!("multirow");
  RequirePackage!("tabularx");
  RequirePackage!("makecell");
  RequirePackage!("array");
  RequirePackage!("colortbl");

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
  DefConstructor!("\\endnote{}",
    "<ltx:note role='endnote'>#1</ltx:note>");
  def_macro_noop("\\printendnotes")?;

  // MDPI frontmatter — preserve author content as ltx:note frontmatter.
  DefMacro!("\\corresref[]{}", "\\textsuperscript{*#1}");
  def_macro_noop("\\externalbibliography{}")?;
  // \firstpage{N} also defines \@firstpage in the real mdpi.cls;
  // some papers reference it via `\setcounter{page}{\@firstpage}`.
  // Witness 2503.04598 — bytedance_seed paper using the mdpi pattern.
  DefMacro!("\\firstpage{}",
    "\\def\\@firstpage{#1}\\@add@frontmatter{ltx:note}[role=firstpage]{#1}");
  DefMacro!("\\@firstpage", "1");
  DefMacro!("\\firstpagenote{}",
    "\\@add@frontmatter{ltx:note}[role=firstpagenote]{#1}");
  DefMacro!("\\corres[]{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#2}");
  DefMacro!("\\Journal{}",
    "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\firstnote{}",
    "\\@add@frontmatter{ltx:note}[role=firstnote]{#1}");
  DefMacro!("\\Address{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#1}");
  DefMacro!("\\AuthorNames{}",
    "\\@add@frontmatter{ltx:note}[role=authornames]{#1}");
  DefMacro!("\\AuthorCitation{}",
    "\\@add@frontmatter{ltx:note}[role=authorcitation]{#1}");
  DefMacro!("\\dates{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=dates]{Received: #1 Revised: #2 Accepted: #3}");
  DefMacro!("\\authorinitials{}",
    "\\@add@frontmatter{ltx:note}[role=authorinitials]{#1}");
  // Additional MDPI frontmatter macros (mdpi.cls L530+). Witness 2412.13512.
  DefMacro!("\\Title{}", "\\title{#1}");
  DefMacro!("\\TitleCitation{}",
    "\\@add@frontmatter{ltx:note}[role=titlecitation]{#1}");
  DefMacro!("\\pubvolume{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\pubyear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\issuenum{}",
    "\\@add@frontmatter{ltx:note}[role=issue]{#1}");
  DefMacro!("\\reftitle{}",
    "\\@add@frontmatter{ltx:note}[role=reftitle]{#1}");
  def_macro_noop("\\PublishersNote")?;
  DefMacro!("\\articlenumber{}",
    "\\@add@frontmatter{ltx:note}[role=articlenumber]{#1}");
  DefMacro!("\\copyrightyear{}",
    "\\@add@frontmatter{ltx:note}[role=copyright-year]{#1}");
  DefMacro!("\\histreceived{}",
    "\\@add@frontmatter{ltx:note}[role=received]{#1}");
  DefMacro!("\\histrevised{}",
    "\\@add@frontmatter{ltx:note}[role=revised]{#1}");
  DefMacro!("\\histaccepted{}",
    "\\@add@frontmatter{ltx:note}[role=accepted]{#1}");
  DefMacro!("\\historypublished{}",
    "\\@add@frontmatter{ltx:note}[role=published]{#1}");
  def_macro_noop("\\SetCaptionDefault")?;

  // Newer mdpi.cls L668-685 — additional date/metadata setters.
  // Witness 2503.11347, 2503.13839 (\daterevised, \datereceived,
  // \dateaccepted).
  DefMacro!("\\datereceived{}",
    "\\@add@frontmatter{ltx:note}[role=received]{#1}");
  DefMacro!("\\daterevised{}",
    "\\@add@frontmatter{ltx:note}[role=revised]{#1}");
  DefMacro!("\\dateaccepted{}",
    "\\@add@frontmatter{ltx:note}[role=accepted]{#1}");
  DefMacro!("\\datepublished{}",
    "\\@add@frontmatter{ltx:note}[role=published]{#1}");
  DefMacro!("\\datecorrected{}",
    "\\@add@frontmatter{ltx:note}[role=corrected]{#1}");
  DefMacro!("\\dateretracted{}",
    "\\@add@frontmatter{ltx:note}[role=retracted]{#1}");
  DefMacro!("\\externaleditor{}",
    "\\@add@frontmatter{ltx:note}[role=external-editor]{#1}");
  DefMacro!("\\LSID{}",
    "\\@add@frontmatter{ltx:note}[role=lsid]{#1}");
  DefMacro!("\\PACS{}",
    "\\@add@frontmatter{ltx:classification}[scheme=PACS]{#1}");
  DefMacro!("\\MSC{}",
    "\\@add@frontmatter{ltx:classification}[scheme=MSC]{#1}");
  DefMacro!("\\JEL{}",
    "\\@add@frontmatter{ltx:classification}[scheme=JEL]{#1}");
  DefMacro!("\\keyword{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}");
  DefMacro!("\\dataset{}",
    "\\@add@frontmatter{ltx:note}[role=dataset]{#1}");
  DefMacro!("\\datasetlicense{}",
    "\\@add@frontmatter{ltx:note}[role=dataset-license]{#1}");

  // Additional newer mdpi.cls macros — preserve content.
  // \Author{name} (capital) is the MDPI variant; route to LaTeX \author.
  DefMacro!("\\Author{}", "\\author{#1}");
  DefMacro!("\\hreflink{}",
    "\\@add@frontmatter{ltx:note}[role=hreflink]{#1}");
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
  DefMacro!("\\authorcontributions{}",
    "\\section*{Author Contributions}#1");
  DefMacro!("\\funding{}",
    "\\section*{Funding}#1");
  DefMacro!("\\conflictsofinterest{}",
    "\\section*{Conflicts of Interest}#1");
  DefMacro!("\\abbreviations{}{}",
    "\\section*{#1}#2");
  // Additional MDPI back-matter macros (mdpi.cls L1199-1240). These are
  // simple `\newcommand` stubs in the raw cls but in practice the cls
  // raw-load stops or truncates partway through, leaving these undefined.
  // Defining here ensures the back-matter content is preserved.
  DefMacro!("\\supplementary{}",
    "\\section*{Supplementary Materials}#1");
  DefMacro!("\\institutionalreview{}",
    "\\section*{Institutional Review Board Statement}#1");
  DefMacro!("\\informedconsent{}",
    "\\section*{Informed Consent Statement}#1");
  DefMacro!("\\dataavailability{}",
    "\\section*{Data Availability Statement}#1");
  DefMacro!("\\publicinvolvement{}",
    "\\section*{Public Involvement Statement}#1");
  DefMacro!("\\guidelinesstandards{}",
    "\\section*{Guidelines and Standards Statement}#1");
  DefMacro!("\\entrylink{}",
    "\\@add@frontmatter{ltx:note}[role=entrylink]{#1}");
  DefMacro!("\\reviewreports{}",
    "\\@add@frontmatter{ltx:note}[role=reviewreports]{#1}");
  // \sampleavailability (older mdpi) — sister of \dataavailability.
  DefMacro!("\\sampleavailability{}",
    "\\section*{Sample Availability}#1");
  // \patents — research patents declaration.
  DefMacro!("\\patents{}",
    "\\section*{Patents}#1");
  // \address[id]{text} — preserve as ltx:note.
  DefMacro!("\\address[]{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#2}");
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
  // \acknowledgments — newer-mdpi spelling (vs \acknowledgements).
  // Render as structural ltx:acknowledgements so post-processors map it
  // to the canonical role/styling (vs flattening to a generic section).
  DefConstructor!("\\acknowledgments{}",
    "<ltx:acknowledgements>#1</ltx:acknowledgements>");
  // \appendixtitles{Yes|No} / \appendixstart — appendix-numbering
  // toggle. No-op (we don't replay mdpi's appendix counter dance).
  def_macro_noop("\\appendixtitles{}")?;
  def_macro_noop("\\appendixstart")?;
});
