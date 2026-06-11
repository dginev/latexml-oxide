// Thanks to Kim Philipp Jablonski <kpjkpjkpjkpjkpjkpj@gmail.com>
// of the arXMLiv group for initial implementation
//    http://arxmliv.kwarc.info/
// Released under the Gnu Public License
// Released to the Public Domain

use crate::prelude::*;

LoadDefinitions!({
  def_macro_noop("\\fancyhead[]{}")?;
  def_macro_noop("\\fancyfoot[]{}")?;
  def_macro_noop("\\fancyhf[]{}")?;

  def_macro_noop("\\fancyheadoffset[]{}")?;
  def_macro_noop("\\fancyfootoffset[]{}")?;
  def_macro_noop("\\fancyhfoffset[]{}")?;

  DefMacro!("\\headrulewidth", "0.4pt");
  DefMacro!("\\footrulewidth", "0pt");
  DefMacro!("\\headruleskip", "0pt"); // since 4.0
  DefMacro!("\\footruleskip", ".3\\normalbaselineskip");
  def_macro_noop("\\headrule")?;
  def_macro_noop("\\footrule")?;
  DefRegister!("\\headwidth" => Dimension(0)); // maybe need some other value here?

  def_macro_noop("\\fancyheadinit{}")?; // since 4.0
  def_macro_noop("\\fancyfootinit{}")?; // since 4.0
  def_macro_noop("\\fancyhfinit{}")?; // since 4.0

  // not implemented yet: \fancycenter[][]{}{}{}, since 4.0

  // always false as LaTeXML does not paginate
  DefMacro!("\\iffloatpage{}{}", "#2");
  DefMacro!("\\iftopfloat{}{}", "#2");
  DefMacro!("\\ifbotfloat{}{}", "#2");
  DefMacro!("\\iffootnote{}{}", "#2"); // since 3.8

  def_macro_noop("\\fancypagestyle{}[]{}")?;

  // extramarks.sty not implemented, as its commands can only be used in headers and footers

  // not defined outside of headers and footers
  // def_macro_noop("\\nouppercase")?;

  // deprecated commands
  def_macro_noop("\\lhead[]{}")?;
  def_macro_noop("\\chead[]{}")?;
  def_macro_noop("\\rhead[]{}")?;

  def_macro_noop("\\lfoot[]{}")?;
  def_macro_noop("\\cfoot[]{}")?;
  def_macro_noop("\\rfoot[]{}")?;

  def_macro_noop("\\fancyplain{}{}")?;

  DefMacro!("\\plainheadrulewidth", "0pt");
  DefMacro!("\\plainfootrulewidth", "0pt");
});
