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
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");
  RequirePackage!("natbib");
  // achemso uses setspace internally; user papers also call
  // \singlespacing / \doublespacing in preambles. Witness 2503.21357.
  RequirePackage!("setspace");
  // achemso.cls L308: `\RequirePackage[margin=2.54cm]{geometry}` and L1306
  // calls `\geometry{...}` for its own layout. The real class (which Perl
  // raw-loads) thus has `\geometry` defined; our OmniBus stub must mirror
  // that so authors' preamble `\geometry{...}` resolves. Layout itself is
  // moot in the XML/HTML paradigm, but the CS must exist. Witness 2407.02650
  // (`\geometry{voffset=10pt,...}` → undefined without this; Perl: 0 errors).
  RequirePackage!("geometry");

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

  // achemso.cls extras commonly hit in ACS papers. The class L1087 sets
  // a section-numbering policy via `\SectionNumbersOn` (preamble only);
  // HTML rendering inherits LaTeX's default numbering so the toggle is
  // a no-op. L294 `\providecommand{\latin}[1]{#1}` is an identity
  // wrapper for italicized Latin abbreviations. Witness 2312.12737.
  DefMacro!("\\SectionNumbersOn", None);
  DefMacro!("\\SectionNumbersOff", None);
  DefMacro!("\\latin{}", "#1");
});
