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
  // Base on OmniBus, NOT article: Perl ships no dmtcs binding and falls back to
  // OmniBus, which provides the generic journal frontmatter (`\received`,
  // `\revised`, `\acknowledgements`/`\acknowledgments`, …). Basing on plain
  // `article` left those undefined where Perl (OmniBus) is clean. The
  // dmtcs-specific setters below are defined AFTER and override OmniBus.
  // Witness 1904.12329 (`\received{}`/`\revised{}`/`\acknowledgements`).
  LoadClass!("OmniBus");
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

  // \affiliation{text} — dmtcs-episciences.cls L251: stores affil text
  // (`\gdef\@affil{#1}`). Preserve as ltx:contact frontmatter.
  // Witness 2403.03614.
  DefMacro!("\\affiliation{}",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");
  DefConstructor!("\\@@@affiliation{}",
    "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  // \affiliationmark{tag} — L229: superscript marker on author/affil.
  def_macro_noop("\\affiliationmark{}")?;
  // \keywords{text} — L311: stores keywords (`\gdef\@keywords{#1}`).
  // Preserve as frontmatter keywords.
  DefMacro!("\\keywords{}",
    "\\@add@frontmatter{ltx:keywords}{#1}");
});
