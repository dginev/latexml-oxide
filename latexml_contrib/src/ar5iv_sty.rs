use latexml_package::prelude::*;

LoadDefinitions!({
  // Perl: PassOptions('latexml', 'sty', ...) + RequirePackage('latexml')
  latexml_core::binding::content::pass_options("latexml", "sty", vec![
    s!("ids"),
    s!("rawstyles"),
    s!("bibconfig=bbl,bib"),
    s!("nobreakuntex"),
    s!("magnify=1.2"),
    s!("zoomout=1.2"),
    s!("tokenlimit=249999999"),
    s!("iflimit=3999999"),
    s!("absorblimit=1299999"),
    s!("pushbacklimit=599999"),
  ])?;
  RequirePackage!("latexml");

  // Practical maximum for warnings
  AssignValue!("MAX_WARNINGS" => 10000i64, Scope::Global);

  // No \today in archival conversions
  RawTeX!(r"\AtBeginDocument{\def\today{\relax}}");
  // TODO: Perl has a DefRewrite that removes non-remote <ltx:resource> elements
  // and monkey-patches LaTeXML::Post::MathML::outerWrapper to add intent=':literal'.
  // These post-processing features are not available in the compile-time binding system.
});
