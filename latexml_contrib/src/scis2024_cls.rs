//! Stub for SCIS2024.cls (Science China Information Sciences 2024).
//!
//! Defines a large set of frontmatter helpers (\ArticleType, \DOI,
//! \Year, \Month, etc.) via `\let\@X\@empty \def\X#1{\def\@X{#1}}`.
//! Our raw-load currently routes to OmniBus instead of the in-archive
//! .cls, leaving every \X CS undefined.
//!
//! Content-preserving stubs: each metadata setter routes its argument
//! into a `ltx:note` frontmatter entry with a role marker, so the
//! author-supplied DOI / year / volume / etc. survives in the XML
//! output rather than being silently dropped.
//!
//! Witness: 2503.01116, 2503.03904 (14 frontmatter undefined cascade).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amsfonts");
  RequirePackage!("amssymb");
  RequirePackage!("bm");
  RequirePackage!("multicol");
  RequirePackage!("mathrsfs");
  RequirePackage!("pifont");
  RequirePackage!("graphicx");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("booktabs");
  RequirePackage!("tabularx");
  RequirePackage!("caption");
  RequirePackage!("subfig");
  RequirePackage!("cite");

  // SCIS2024 metadata setters — preserve the author-supplied value as
  // a ltx:note frontmatter entry rather than discarding it.
  DefMacro!(
    "\\ArticleType{}",
    "\\@add@frontmatter{ltx:note}[role=articletype]{#1}"
  );
  DefMacro!(
    "\\SpecialTopic{}",
    "\\@add@frontmatter{ltx:note}[role=specialtopic]{#1}"
  );
  DefMacro!("\\Year{}", "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\Month{}", "\\@add@frontmatter{ltx:note}[role=month]{#1}");
  DefMacro!("\\Vol{}", "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\No{}", "\\@add@frontmatter{ltx:note}[role=number]{#1}");
  DefMacro!(
    "\\AuthorMark{}",
    "\\@add@frontmatter{ltx:note}[role=authormark]{#1}"
  );
  DefMacro!(
    "\\AuthorCitation{}",
    "\\@add@frontmatter{ltx:note}[role=authorcitation]{#1}"
  );
  DefMacro!(
    "\\BeginPage{}",
    "\\@add@frontmatter{ltx:note}[role=startpage]{#1}"
  );
  DefMacro!(
    "\\EndPage{}",
    "\\@add@frontmatter{ltx:note}[role=endpage]{#1}"
  );
  DefMacro!("\\DOI{}", "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!(
    "\\ArtNo{}",
    "\\@add@frontmatter{ltx:note}[role=articleno]{#1}"
  );
  DefMacro!(
    "\\ReceiveDate{}",
    "\\@add@frontmatter{ltx:note}[role=received]{#1}"
  );
  DefMacro!(
    "\\ReviseDate{}",
    "\\@add@frontmatter{ltx:note}[role=revised]{#1}"
  );
  DefMacro!(
    "\\AcceptDate{}",
    "\\@add@frontmatter{ltx:note}[role=accepted]{#1}"
  );
  DefMacro!(
    "\\OnlineDate{}",
    "\\@add@frontmatter{ltx:note}[role=onlinedate]{#1}"
  );
  DefMacro!(
    "\\contributions{}",
    "\\@add@frontmatter{ltx:note}[role=contributions]{#1}"
  );
  def_macro_noop("\\luntan")?;
  def_macro_noop("\\oa")?;
  // \Acknowledgements opens an acknowledgements section. Content
  // following it is the body, which is preserved naturally.
  DefMacro!("\\Acknowledgements", "\\section*{Acknowledgements}");
});
