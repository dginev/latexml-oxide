//! Stub for achemso.cls (ACS chemistry journals).
//!
//! achemso.cls is an article-derivative for ACS journals. Provides
//! authorship/affiliation primitives (\affiliation, \alsoaffiliation,
//! \altaffiliation, \email, \phone, \fax). Gobble for now since we
//! don't render ACS-style title pages.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("natbib");
  // achemso uses setspace internally; user papers also call
  // \singlespacing / \doublespacing in preambles. Witness 2503.21357.
  RequirePackage!("setspace");

  // ACS authorship primitives — preserve author content as ltx:note
  // frontmatter entries.
  DefMacro!("\\affiliation[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}");
  DefMacro!("\\alsoaffiliation[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}");
  DefMacro!("\\altaffiliation[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}");
  DefMacro!("\\email{}",
    "\\@add@frontmatter{ltx:note}[role=email]{#1}");
  DefMacro!("\\phone{}",
    "\\@add@frontmatter{ltx:note}[role=phone]{#1}");
  DefMacro!("\\fax{}",
    "\\@add@frontmatter{ltx:note}[role=fax]{#1}");
  DefMacro!("\\suppinfo{}",
    "\\@add@frontmatter{ltx:note}[role=suppinfo]{#1}");
  DefMacro!("\\manuscript{}",
    "\\@add@frontmatter{ltx:note}[role=manuscript]{#1}");
  DefMacro!("\\abbreviations{}",
    "\\@add@frontmatter{ltx:note}[role=abbreviations]{#1}");
  // \acsAuthorList — emit the author-list text inline (no frontmatter slot).
  DefMacro!("\\acsAuthorList{}", "#1");
  DefMacro!("\\notetext{}",
    "\\@add@frontmatter{ltx:note}[role=notetext]{#1}");
  // \acsSection — section opener with text becoming heading.
  DefMacro!("\\acsSection{}", "\\section*{#1}");

  // {tocentry} environment — table of contents image, suppress.
  DefMacro!(T_CS!("\\begin{tocentry}"), None, "\\iffalse");
  DefMacro!(T_CS!("\\end{tocentry}"), None, "\\fi");

  // {acknowledgement} — ACS-spelt acknowledgement section.
  DefEnvironment!(
    "{acknowledgement}",
    "<ltx:acknowledgements>#body</ltx:acknowledgements>"
  );
});
