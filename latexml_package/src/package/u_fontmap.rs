//! The `U` ("unknown") font encoding: glyphs come from the FAMILY. A raw
//! `\DeclareSymbolFont{AMSa}{U}{msa}{m}{n}` (oz.sty:34, amsfonts users) then
//! decodes through the per-family map `U_msa_fontmap` (`font_decode`; AMSb's `msb` through
//! `U_msb_fontmap`, which amsfonts' `\newsymbol` reaches), which
//! Perl (no `u.fontmap.ltxml`) leaves undecoded. The encoding has no map of its own: a U symbol font of another family
//! (shuffle.sty, stmaryrd's `stmry`) stays undecoded — but for a kernel or AMS name declared at its fontmath.ltx or
//! amssymb slot (sect08.rs `reference_symbol`, OXIDIZED_DESIGN_DIVERGENCES #910) — and the text switch
//! `\fontencoding{U}` falls back to OT1 while `\f@encoding` still names `U` (`\lx@fontencoding`, tex_fonts.rs;
//! OXIDIZED_DESIGN_DIVERGENCES #645). Guards:
//! `perfect_kernel_batch54::declare_math_delimiter_defines_the_symbol`,
//! `perfect_kernel_batch68v::u_symbol_font_without_a_map_stays_undecoded`.
//! Witness: 2306.04672 (`\shuffle`).
use crate::prelude::*;

LoadDefinitions!({
  DeclareFontMap!("U", amsa_fontmap::amsa_table(), "msa");
  DeclareFontMap!("U", amsb_fontmap::amsb_table(), "msb");
});
