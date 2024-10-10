// Thanks to Kim Philipp Jablonski <kpjkpjkpjkpjkpjkpj@gmail.com>
// of the arXMLiv group for initial implementation
//    http://arxmliv.kwarc.info/
// Released under the Gnu Public License
// Released to the Public Domain

use crate::prelude::*;

LoadDefinitions!({
  DefMacro!("\\fancyhead[]{}", "");
  DefMacro!("\\fancyfoot[]{}", "");
  DefMacro!("\\fancyhf[]{}", "");

  DefMacro!("\\fancyheadoffset[]{}", "");
  DefMacro!("\\fancyfootoffset[]{}", "");
  DefMacro!("\\fancyhfoffset[]{}", "");

  DefMacro!("\\headrulewidth", "0.4pt");
  DefMacro!("\\footrulewidth", "0pt");
  DefMacro!("\\headruleskip", "0pt"); // since 4.0
  DefMacro!("\\footruleskip", ".3\\normalbaselineskip");
  DefMacro!("\\headrule", "");
  DefMacro!("\\footrule", "");
  DefRegister!("\\headwidth" => Dimension(0)); // maybe need some other value here?

  DefMacro!("\\fancyheadinit{}", ""); // since 4.0
  DefMacro!("\\fancyfootinit{}", ""); // since 4.0
  DefMacro!("\\fancyhfinit{}", ""); // since 4.0

  // not implemented yet: \fancycenter[][]{}{}{}, since 4.0

  // always false as LaTeXML does not paginate
  DefMacro!("\\iffloatpage{}{}", "#2");
  DefMacro!("\\iftopfloat{}{}", "#2");
  DefMacro!("\\ifbotfloat{}{}", "#2");
  DefMacro!("\\iffootnote{}{}", "#2"); // since 3.8

  DefMacro!("\\fancypagestyle{}[]{}", "");

  // extramarks.sty not implemented, as its commands can only be used in headers and footers

  // not defined outside of headers and footers
  // DefMacro!("\\nouppercase", "");

  // deprecated commands
  DefMacro!("\\lhead[]{}", "");
  DefMacro!("\\chead[]{}", "");
  DefMacro!("\\rhead[]{}", "");

  DefMacro!("\\lfoot[]{}", "");
  DefMacro!("\\cfoot[]{}", "");
  DefMacro!("\\rfoot[]{}", "");

  DefMacro!("\\fancyplain{}{}", "");

  DefMacro!("\\plainheadrulewidth", "0pt");
  DefMacro!("\\plainfootrulewidth", "0pt");
});
