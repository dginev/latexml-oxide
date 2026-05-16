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

  // Frontmatter / paper metadata
  DefMacro!("\\TITLE{}", "");
  DefMacro!("\\RUNAUTHOR{}", "");
  DefMacro!("\\RUNTITLE{}", "");
  DefMacro!("\\ECRUNAUTHOR{}", "");
  DefMacro!("\\ECRUNTITLE{}", "");
  DefMacro!("\\AUTHOR{}{}", "");
  DefMacro!("\\AFF[]{}", "");
  DefMacro!("\\ABSTRACT{}", "");
  DefMacro!("\\KEYWORDS{}", "");
  DefMacro!("\\MANUSCRIPTNO{}", "");
  DefMacro!("\\HISTORY{}", "");
  DefMacro!("\\ARTICLEAUTHORS{}", "");
  DefMacro!("\\ARTICLETITLE{}", "");
  DefMacro!("\\ARTICLEABSTRACT{}", "");
  DefMacro!("\\authorinfo{}", "");
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
