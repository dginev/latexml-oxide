//! Stub for INFORMS journal classes (informs, informs3).
//!
//! The informs* classes are used by Operations Research, Management Science,
//! and other INFORMS journals. They define a large frontmatter API
//! (\TITLE, \ARTICLEAUTHORS, \ABSTRACT, \KEYWORDS, etc.) inside the cls,
//! which we never raw-load. Provide gobble stubs so papers using this
//! class convert without "undefined" cascades.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");

  // Frontmatter / paper metadata — preserve author content.
  // \TITLE / \ARTICLETITLE → \title to populate the document title.
  DefMacro!("\\TITLE{}", "\\title{#1}");
  DefMacro!("\\ARTICLETITLE{}", "\\title{#1}");
  // Running header variants — short title for header; preserve as note.
  DefMacro!("\\RUNAUTHOR{}",
    "\\@add@frontmatter{ltx:note}[role=runningauthor]{#1}");
  DefMacro!("\\RUNTITLE{}",
    "\\@add@frontmatter{ltx:note}[role=runningtitle]{#1}");
  DefMacro!("\\ECRUNAUTHOR{}",
    "\\@add@frontmatter{ltx:note}[role=ec-runningauthor]{#1}");
  DefMacro!("\\ECRUNTITLE{}",
    "\\@add@frontmatter{ltx:note}[role=ec-runningtitle]{#1}");
  // \AUTHOR{name}{affiliation} — emit name as author, affiliation as note.
  DefMacro!("\\AUTHOR{}{}",
    "\\author{#1}\\@add@frontmatter{ltx:note}[role=affiliation]{#2}");
  DefMacro!("\\AFF[]{}",
    "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}");
  // \ABSTRACT → abstract env so the text is preserved as document abstract.
  DefMacro!("\\ABSTRACT{}",
    "\\begin{abstract}#1\\end{abstract}");
  DefMacro!("\\ARTICLEABSTRACT{}",
    "\\begin{abstract}#1\\end{abstract}");
  DefMacro!("\\KEYWORDS{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}");
  DefMacro!("\\MANUSCRIPTNO{}",
    "\\@add@frontmatter{ltx:note}[role=manuscriptno]{#1}");
  DefMacro!("\\HISTORY{}",
    "\\@add@frontmatter{ltx:note}[role=history]{#1}");
  DefMacro!("\\ARTICLEAUTHORS{}",
    "\\@add@frontmatter{ltx:note}[role=authors]{#1}");
  DefMacro!("\\authorinfo{}",
    "\\@add@frontmatter{ltx:note}[role=authorinfo]{#1}");
  DefMacro!("\\thetitle", "");

  // Layout / structure switches — no visual effect in XML.
  DefMacro!("\\TheoremsNumberedThrough", "");
  DefMacro!("\\TheoremsNumberedBySection", "");
  DefMacro!("\\EquationsNumberedThrough", "");
  DefMacro!("\\EquationsNumberedBySection", "");
  DefMacro!("\\ECRepeatTheorems", "");
  DefMacro!("\\OneAndAHalfSpacedXI", "");
  DefMacro!("\\OneAndAHalfSpacedXII", "");
  DefMacro!("\\DoubleSpacedXI", "");
  DefMacro!("\\DoubleSpacedXII", "");
  DefMacro!("\\SingleSpacedXI", "");

  // {APPENDICES} env — render contents as appendix section.
  DefMacro!(T_CS!("\\begin{APPENDICES}"), None, "\\appendix");
  DefMacro!(T_CS!("\\end{APPENDICES}"), None, "");
});
