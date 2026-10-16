//! The `U` ("unknown") font encoding: glyphs come from the FAMILY. A raw
//! `\DeclareSymbolFont{AMSa}{U}{msa}{m}{n}` (oz.sty:34, amsfonts users) then
//! decodes through the per-family map `U_msa_fontmap` (`font_decode`; AMSb's `msb` through
//! `U_msb_fontmap`, which amsfonts' `\newsymbol` reaches), which
//! Perl (no `u.fontmap.ltxml`) leaves undecoded; any other family decodes as OT1. Guard:
//! `perfect_kernel_batch54::declare_math_delimiter_defines_the_symbol`.
use crate::prelude::*;

LoadDefinitions!({
  // The encoding's own map, for a family without one: OT1's, the glyphs `\fontencoding{U}` decoded through when it fell
  // back to OT1 for want of a map ("Couldn't find font encoding"). With it the encoding stays `U`, so `\f@encoding`
  // names it as latex.ltx's `\fontencoding` does (:10490), and `\fontencoding{U}\fontfamily{msa}\try@load@fontshape`
  // finds umsa.fd — amsart-derived classes load amsfonts on that test (amsart.cls:338-350; nyjm.cls 2401.06615,
  // cedram.cls 2105.03188, myart.cls 1506.02470: `\symAMSb` was undefined). OXIDIZED_DESIGN_DIVERGENCES #645.
  if let Some(ot1) = load_font_map("OT1") {
    DeclareFontMap!("U", ot1);
  }
  DeclareFontMap!("U", amsa_fontmap::amsa_table(), "msa");
  DeclareFontMap!("U", amsb_fontmap::amsb_table(), "msb");
});
