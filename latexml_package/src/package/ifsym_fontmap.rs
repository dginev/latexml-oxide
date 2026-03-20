//! ifsym font encoding (from ifsym.fontmap.ltxml)
//! Various symbols: mountains, dice, clocks, etc.
use crate::prelude::*;

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("ifsym", mixrc![
    // 0x00-0x07
    '\u{2709}',  None,         '\u{2756}',  '\u{1F5B9}',
    None,         '\u{2680}',  '\u{2681}',  '\u{2682}',
    // 0x08-0x0F
    '\u{2683}',  '\u{2684}',  '\u{2685}',  '\u{274C}',
    '\u{1F525}', None,         '\u{2622}',  None,
    // 0x10-0x17
    None,         None,         '\u{1F6D6}', '\u{1F6D6}',
    None,         '\u{26F0}',  '\u{1F3D4}', '\u{26F0}',
    // 0x18-0x1F
    '\u{26F0}',  None,         None,         '\u{1F6A9}',
    '\u{26FA}',  '\u{2691}',  None,         '\u{1F6D6}',
    // 0x20-0x27: stairstep, pulse, etc — undef in Perl
    None,         None,         None,         None,
    None,         None,         None,         None,
    // 0x28-0x2F
    '\u{260E}',  '\u{1F805}', '\u{1F807}', None,
    None,         '-',          None,         None,
    // 0x30-0x37
    '\u{1FBF0}', '\u{1FBF1}', '\u{1FBF2}', '\u{1FBF3}',
    '\u{1FBF4}', '\u{1FBF5}', '\u{1FBF6}', '\u{1FBF7}',
    // 0x38-0x3B
    '\u{1FBF8}', '\u{1FBF9}', '\u{1D377}', '\u{1D378}'
  ]);
});
