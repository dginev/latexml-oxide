//! The `U` ("unknown") font encoding: glyphs come from the FAMILY. A raw
//! `\DeclareSymbolFont{AMSa}{U}{msa}{m}{n}` (oz.sty:34, amsfonts users) then
//! decodes through the per-family map `U_msa_fontmap` (`font_decode`), which
//! Perl (no `u.fontmap.ltxml`) leaves undecoded. Guard:
//! `perfect_kernel_batch54::declare_math_delimiter_defines_the_symbol`.
use crate::prelude::*;

LoadDefinitions!({
  DeclareFontMap!("U", amsa_fontmap::amsa_table(), "msa");
});
