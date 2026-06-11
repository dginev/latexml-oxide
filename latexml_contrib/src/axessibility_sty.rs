use latexml_package::prelude::*;

LoadDefinitions!({
  // This package targets Tagged PDF and is largely a no-op from a LaTeXML standpoint.
  DeclareOption!("accsupp", "");
  DeclareOption!("tagpdf", "");
  ProcessOptions!();
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("xstring");
  DefConditional!("\\iftagpdfopt", { false });
  DefMacro!("\\auxiliaryspace", " ");
  DefMacro!("\\wrap{}", "#1");
  DefMacro!("\\wrapml{}", "#1");
  DefMacro!("\\wrapmlalt{}", "#1");
  DefMacro!("\\wrapmlstar{}", "#1");
  def_macro_noop("\\doreplacement{}")?;
  DefEnvironment!("{tempenv}", "#body");
});
