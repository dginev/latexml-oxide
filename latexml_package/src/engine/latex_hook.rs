use crate::prelude::*;

LoadDefinitions!({
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
  // TODO: Port and use `DefAutoload` instead of this single-purpose macro
  DefPrimitive!("\\@load@latex@pool", {
    input_definitions(
      "LaTeX",
      InputDefinitionOptions {
        extension: Some(Cow::Borrowed("pool")),
        ..InputDefinitionOptions::default()
      },
    )?;
  });

  for _ltx3trigger in [
    "\\ExplSyntaxOn",
    "\\ProvidesExplClass",
    "\\ProvidesExplPackage",
  ] {
    // DG: note that these auto-loads are not perfect --
    //     if they are triggered with a raw .sty file for example,
    //     the expl3 support will "expire" at the end of the current scope,
    //     and e.g. \ExplSyntaxOn will once again be undefined.
    // TODO:
    // DefAutoload!(ltx3trigger, "expl3.pool.ltxml");
  }

  // # Darn; we need to be even more clever, since we need to simulate an amstex command, as well.
  // # For example \documentstyle[...]{amsppt} must switch to AMSTeX mode, _NOT_ LaTeX mode!!!!
  // DefMacro('\documentstyle OptionalSemiverbatim SkipSpaces Semiverbatim', sub {
  //   my ($gullet, $options, $class) = @_;
  //   LoadPool((ToString($class) =~ /^amsppt$/ ? "AmSTeX" : "LaTeX"));
  //   (T_CS('\\documentstyle'),
  //     ($options ? (T_OTHER('['), $options->unlist, T_OTHER(']')) : ()),
  //     T_BEGIN, $class->unlist, T_END); });

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
  Let!("\\@currname", "\\lx@empty");
  Let!("\\@currext", "\\lx@empty");
});
