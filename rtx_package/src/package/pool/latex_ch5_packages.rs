use crate::package::*;
lazy_static! {
  static ref OPTS_REGEX: Regex = Regex::new(r",\s*").unwrap();
}

LoadDefinitions!(state, {
  // ======================================================================
  // C.5.2 Packages
  // ======================================================================
  // We'll prefer to load package.pm, but will try package.sty or
  // package.tex (the latter being unlikely to work, but....)
  // See Stomach.pm for details
  // Ignorable packages ??
  // pre-defined packages??

  DefMacro!("\\@clsextension", "cls");
  DefMacro!("\\@pkgextension", "sty");
  Let!("\\@currext", "\\@empty");
  Let!("\\@currname", "\\@empty");

  DefConstructor!("\\usepackage OptionalSemiverbatim Semiverbatim []",
                  "<?latexml package='#2' ?#1(options='#1')?>",
      before_digest => before_digest!(_stomach, state, { only_preamble("\\usepackage", state); }),
      after_digest => sub!(|stomach: &mut Stomach, whatsit: &mut Whatsit, state: &mut State| -> Result<Vec<Digested>> {
        let options: Option<&Digested> = whatsit.get_arg(1);
        let packages: Option<&Digested> = whatsit.get_arg(2);
        let package_list = match packages {
          Some(value) => OPTS_REGEX.split(&value.to_string()).map(ToString::to_string).filter(|s| !s.starts_with('%')).collect(),
          None => Vec::new(),
        };
        let options_list = match options {
          Some(opts) => OPTS_REGEX.split(&opts.to_string()).map(ToString::to_string).collect(),
          None => Vec::new(),
        };
        for package in package_list {
          require_package(&package, RequireOptions {
            options: options_list.clone(),
            ..RequireOptions::default()
          }, stomach, state)?
        }
        Ok(Vec::new())
      })
  );

  // STUBS:
  for ltxtrigger in [
    "\\renewcommand",
    "\\NeedsTeXFormat",
    "\\ProvidesPackage",
    "\\RequirePackage",
    "\\ProvidesFile",
    "\\makeatletter",
    "\\makeatother",
    "\\typeout",
    "\\listfiles",
  ]
  .iter()
  .map(ToString::to_string)
  {
    DefMacroI!(T_CS!(ltxtrigger), None, Tokens!());
  }
});
