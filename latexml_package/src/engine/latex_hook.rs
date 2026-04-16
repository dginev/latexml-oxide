use crate::prelude::*;

/// Perl: DefAutoload — define a macro that auto-loads a package on first use.
/// When the command is first invoked, it loads the specified package (via RequirePackage),
/// then re-emits the original CS so it gets re-executed with the proper definition.
fn def_autoload(cs_name: &str, package: &str) -> Result<()> {
  use latexml_core::definition::ExpansionBody;
  let cs_tok = T_CS!(cs_name);
  // Don't overwrite if already defined
  if IsDefined!(&cs_tok) {
    return Ok(());
  }
  let pkg_name = package.to_string();
  let cs_for_closure = cs_tok;
  def_macro(
    cs_tok,
    None,
    ExpansionBody::Closure(Rc::new(move |_args| {
      require_package(&pkg_name, RequireOptions::default())?;
      Ok(Tokens::new(vec![cs_for_closure]))
    })),
    None,
  )?;
  Ok(())
}

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
    "\\nofiles",
    "\\typeout",
    "\\PassOptionsToPackage",
  ]
  .iter()
  {
    DefMacro!(T_CS!(ltxtrigger), None, {
      Tokens!(T_CS!("\\@load@latex@pool"), T_CS!(ltxtrigger))
    });
  }
  // TODO: Port and use `DefAutoload` instead of this single-purpose macro
  DefPrimitive!("\\@load@latex@pool", {
    input_definitions("LaTeX", InputDefinitionOptions {
      extension: Some(Cow::Borrowed("pool")),
      ..InputDefinitionOptions::default()
    })?;
  });

  // Perl TeX.pool.ltxml L42-48: DefAutoload for expl3 triggers.
  // When \ExplSyntaxOn (etc.) is encountered without expl3 loaded,
  // auto-load expl3.sty, then re-emit the trigger command.
  // Note: these auto-loads are not perfect — if triggered from a raw
  // .sty file, the expl3 support may expire at end of current scope.
  for ltx3trigger in [
    "\\ExplSyntaxOn",
    "\\ProvidesExplClass",
    "\\ProvidesExplPackage",
  ] {
    def_autoload(ltx3trigger, "expl3")?;
  }

  // OmniBus autoloads: define commands that auto-load their packages on first use.
  // Perl: OmniBus.cls.ltxml DefAutoload entries.
  // When first invoked, the macro loads the package, then re-emits itself.
  // amsfonts
  def_autoload("\\mathfrak", "amsfonts")?;
  def_autoload("\\mathbb", "amsfonts")?;
  def_autoload("\\Bbb", "amsfonts")?;
  // amsthm
  def_autoload("\\theoremstyle", "amsthm")?;
  // amsmath
  def_autoload("\\numberwithin", "amsmath")?;
  def_autoload("\\align", "amsmath")?;
  def_autoload("\\subequations", "amsmath")?;
  def_autoload("\\multline", "amsmath")?;
  // ams_support (AMS article metadata)
  def_autoload("\\curraddr", "ams_support")?;
  def_autoload("\\subjclass", "ams_support")?;

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

  // Early stubs needed by ProcessOptions/DeclareOption before LaTeX.pool loads.
  // These are normally in latex_ch5_packages.rs but must exist when --preload
  // directives invoke RequirePackage (e.g., ar5iv.sty → latexml.sty).
  // LaTeX.pool will provide full definitions; these are just no-op placeholders.
  if !IsDefined!(&T_CS!("\\@unknownoptionerror")) {
    DefPrimitive!("\\@unknownoptionerror", {});
  }
  if !IsDefined!(&T_CS!("\\OptionNotUsed")) {
    DefPrimitive!("\\OptionNotUsed", {});
  }
  if !IsDefined!(&T_CS!("\\AtBeginDocument")) {
    // Stub: collect hooks to be executed later when \begin{document} runs.
    // LaTeX.pool provides the real implementation.
    DefMacro!("\\AtBeginDocument{}", "");
  }
  if !IsDefined!(&T_CS!("\\@addtofilelist")) {
    DefMacro!("\\@addtofilelist{}", "");
  }
});
