use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: xunicode.sty.ltxml
  // Preliminary support for xelatex
  AssignValue!("PERL_INPUT_ENCODING" => "utf8");
  RequirePackage!("textcomp");

  DefMacro!("\\UTFencname", "TU");
  def_macro_noop("\\ReloadXunicode{}")?;
  def_macro_noop("\\UseMathAsText")?;
  def_macro_noop("\\DeclareUTFcharacter[]{}{}")?;
  def_macro_noop("\\UndeclareUTFcharacter[]{}{}")?;
  def_macro_noop("\\DeclareUTFcomposite[]{}{}{}")?;
  def_macro_noop("\\DeclareUTFmulticomposite[]{}{}{}")?;
  def_macro_noop("\\UndeclareUTFcomposite[]{}{}{}")?;
  def_macro_noop("\\DeclareMathAsUTFtext{}{}{}")?;
  def_macro_noop("\\DeclareUTFmathsymbols{}")?;
  def_macro_noop("\\DeclareEncodedCompositeCharacter{}{}{}{}")?;
  def_macro_noop("\\DeclareEncodedCompositeAccents{}{}{}{}")?;
});
