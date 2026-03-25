use latexml_package::prelude::*;

LoadDefinitions!({
  // TODO: Perl uses PassOptions to latexml.sty with ids, rawstyles, bibconfig, magnify, etc.
  // PassOptions! is not available yet. Load latexml.sty directly for now.
  RequirePackage!("latexml");
  // TODO: Perl sets AssignValue('MAX_WARNINGS', 10_000, 'global')
  // TODO: Perl has AtBeginDocument to redefine \today as \relax
  DefMacro!("\\today", "\\relax");
  // TODO: Perl has a DefRewrite that removes non-remote <ltx:resource> elements
  // and monkey-patches LaTeXML::Post::MathML::outerWrapper to add intent=':literal'.
  // These post-processing features are not available in the compile-time binding system.
});
