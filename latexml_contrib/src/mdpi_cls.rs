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

  // MDPI frontmatter — preserve author content as ltx:note frontmatter.
  DefMacro!("\\corresref[]{}", "\\textsuperscript{*#1}");
  DefMacro!("\\externalbibliography{}", "");
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
  DefMacro!("\\PublishersNote", "");
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
  DefMacro!("\\SetCaptionDefault", "");

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
  DefMacro!("\\orcidA", "");
  DefMacro!("\\orcidB", "");
  DefMacro!("\\orcidC", "");
  DefMacro!("\\orcidD", "");
  DefMacro!("\\orcidE", "");
  DefMacro!("\\orcidF", "");
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
  // \address[id]{text} — preserve as ltx:note.
  DefMacro!("\\address[]{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#2}");
  // \natexlab from natbib — emit arg inline (used in \bibitem to mark
  // companion years like (1999a)/(1999b)).
  DefMacro!("\\natexlab{}", "#1");
  // \textls (microtype letterspacing) — emit as-is.
  DefMacro!("\\textls[]{}", "#2");
});
