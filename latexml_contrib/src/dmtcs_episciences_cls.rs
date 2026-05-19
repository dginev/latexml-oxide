//! Stub for dmtcs-episciences.cls (Discrete Mathematics & Theoretical
//! Computer Science journal class, Episciences platform).
//!
//! The raw cls preamble has font/page-layout machinery and conditional
//! `\@firsthead` formatting (real `\publicationdata` builds running
//! headers via tikz). Mid-load fails before reaching the publication
//! macros, leaving `\publicationdata`, `\papertype`, `\fundinginfo`,
//! `\accepted` undefined. Witness 2309.05874, 2309.12265.
use latexml_package::prelude::*;


LoadDefinitions!({
  LoadClass!("article");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("graphicx");
  RequirePackage!("hyperref");

  // \publicationdata{volume}{year}{issue}{doi}{dates}{accepted} —
  // 6-arg setter for running headers. Preserve as frontmatter note.
  DefMacro!("\\publicationdata{}{}{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=dmtcs-publicationdata]{Volume #1 (#2), \\##3, doi:#4}");
  // \papertype{T} — single-arg paper-type marker (Article/Editorial/…).
  DefMacro!("\\papertype{}",
    "\\@add@frontmatter{ltx:note}[role=papertype]{#1}");
  // \fundinginfo{text} — funding statement.
  DefMacro!("\\fundinginfo{}",
    "\\@add@frontmatter{ltx:note}[role=funding]{#1}");
  // \accepted{date} — accepted-date setter.
  DefMacro!("\\accepted{}",
    "\\@add@frontmatter{ltx:note}[role=accepted]{Accepted: #1}");
  // \processdates / \endprocessdates — internal parser hooks; no-op.
  def_macro_noop("\\processdates")?;
  def_macro_noop("\\endprocessdates")?;
  def_macro_noop("\\getinfo")?;
});
