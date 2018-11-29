use package::*;

pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

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
    "\\typeout",
    "\\begin",
    "\\listfiles",
  ]
    .iter()
    .map(|s| s.to_string())
  {
    let inner_ltxtrigger = ltxtrigger.clone();
    DefMacroI!(T_CS!(ltxtrigger), None, sub[ _gullet, _args, state] {
      input_definitions(
        "LaTeX",
        InputDefinitionOptions {
          extension: Some(String::from("pool")),
          ..InputDefinitionOptions::default()
        },
        state,
      )?;
      Ok(Tokens!(T_CS!(inner_ltxtrigger)))
    });
  }

  Ok(())
}
