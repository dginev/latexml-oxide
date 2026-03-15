//! ifwea font encoding (from ifwea.fontmap.ltxml)
//! Weather symbols.
use crate::prelude::*;

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("ifwea", mixrc![
    // 0x00-0x07: moon phases and thermometers
    '\u{25CB}',  '\u{25D4}',  '\u{25D3}',  '\u{25D3}',
    '\u{25CF}',  '\u{1F321}', '\u{1F321}', '\u{1F321}',
    // 0x08-0x0F: thermometers continued
    '\u{1F321}', '\u{1F321}', '\u{1F321}', '\u{1F321}',
    None,         None,         None,         None,
    // 0x10-0x17: sun, fog, haze, snow
    '\u{1F323}', None,         None,         '\u{1F32B}',
    '\u{1F32B}', '\u{26C6}',  '\u{26C6}',  None,
    // 0x18-0x1F: snowflake, clouds, rain
    None,         '\u{2744}',  '\u{1F5F2}', '\u{2601}',
    '\u{1F327}', '\u{1F327}', '\u{1F324}', '\u{1F328}',
    // 0x20-0x27: clouds black variants
    '\u{2601}',  '\u{1F327}', '\u{1F327}', '\u{1F324}',
    '\u{1F328}', None,         None,         None,
    // 0x28-0x2F
    None,         None,         None,         None,
    None,         None,         None,         None,
    // 0x30-0x37: musical flags area (undef in Perl)
    None,         None,         None,         None,
    None,         None,         None,         None
  ]);
});
