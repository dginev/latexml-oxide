use latexml_package::prelude::*;

LoadDefinitions!({
  // hypdestopt optimizes PDF hyperlink destination names. In LaTeXML,
  // hyperref destinations are represented directly in XML/HTML, so PDF
  // destination optimization is out-of-scope and a no-op.
  DeclareOption!("verbose", "");
  DeclareOption!("num", "");
  DeclareOption!("name", "");
  ProcessOptions!();
  RequirePackage!("hyperref");
  DefMacro!("\\HypDest@VerboseInfo{}", "");
});
