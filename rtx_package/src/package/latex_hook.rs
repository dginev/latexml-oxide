use crate::package::*;

LoadDefinitions!(state, {
  //**********************************************************************
  // LaTeX Hook
  //**********************************************************************
  // This is used for plain TeX, but needs to be undone for LaTeX (or...)!
  RelaxNGSchema!("LaTeXML");
  Tag!("ltx:section", auto_close => true);
  Tag!("ltx:document", auto_close => true, auto_open => true);
  // TODO:
  // Tag("ltx:document", after_open => sub {
  //     my ($document, $root) = @_;
  //     if (my $font = $document->getNodeFont($root)) {
  //       if (my $bg = $font->getBackground) {
  //         if ($bg ne 'white') {
  //           $document->setAttribute($root, backgroundcolor => $bg); } } } });

  // No, \documentclass isn't really a primitive -- It's not even TeX!
  // But we define a number of stubs here that will automatically load
  // the LaTeX pool (or AmSTeX.pool) (which will presumably redefine them), and then
  // stuff the token back to be reexecuted.
  for ltxtrigger in [
    "\\documentclass",
    "\\newcommand",
    "\\renewcommand",
    "\\newenvironment",
    "\\renewenvironment",
    "\\NeedsTeXFormat",
    "\\ProvidesPackage",
    "\\RequirePackage",
    "\\ProvidesFile",
    "\\makeatletter",
    "\\makeatother",
    "\\begin",
    "\\listfiles",
  ]
  .iter()
  {
    DefMacro!(T_CS!(ltxtrigger), None, {
      Tokens!(T_CS!("\\@load@latex@pool"), T_CS!(ltxtrigger))
    });
  }

  DefPrimitive!("\\@load@latex@pool", sub[stomach, (), state] {
    input_definitions(
      "LaTeX",
      InputDefinitionOptions {
        extension: Some(Cow::Borrowed("pool")),
        ..InputDefinitionOptions::default()
      },
      // Note: passing in "stomach" is crucial,
      // or we can't invoke any RawTeX-like macros in the pool
      // due to multiple mutable borrows of stomach!
      stomach,
      state,
    )?;
  });

  // Technically should be in LaTeX.pool, but we try to maintain the bookkeeping from the very
  // start, in order to avoid partially defined behavior when --preload directives are mixed with
  // \usepackage{} loads
  DefMacro!(
    "\\@pushfilename",
    r"\xdef\@currnamestack{{\@currname}{\@currext}{\the\catcode`\@}\@currnamestack}"
  );
  DefMacro!(
    "\\@popfilename",
    r"\expandafter\@p@pfilename\@currnamestack\@nil"
  );
  DefMacro!(
    "\\@p@pfilename {}{}{} Until:\\@nil",
    r"\gdef\@currname{#1}%
      \gdef\@currext{#2}%
      \catcode`\@#3\relax
      \gdef\@currnamestack{#4}"
  );
  DefMacro!(T_CS!("\\@currnamestack"), None, Tokens!());
});
