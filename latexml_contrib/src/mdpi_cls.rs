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
  DefMacro!("\\firstpage{}",
    "\\@add@frontmatter{ltx:note}[role=firstpage]{#1}");
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
});
