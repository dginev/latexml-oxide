//! ifgeo font encoding (from ifgeo.fontmap.ltxml)
//! Geometric shapes.
use crate::prelude::*;

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("ifgeo", mixrc![
    // 0x00-0x07: flower outlines (same?)
    '\u{274F}', '\u{274F}', '\u{274F}', '\u{274F}',
    '\u{274F}', None,        None,        None,
    // 0x08-0x0F
    None,        None,        None,        None,
    None,        '\u{274C}', '\u{274C}', '\u{274C}',
    // 0x10-0x17
    None,        None,        None,        None,
    None,        None,        None,        None,
    // 0x18-0x1F: dashes
    None,        None,        '\u{2014}', '\u{2013}',
    '\u{2012}', '\u{FE31}', '\u{FE32}', None,
    // 0x20-0x27: large, white shapes
    '\u{25A1}', '\u{25B3}', '\u{25C1}', '\u{25BD}',
    '\u{25B7}', '\u{25EF}', '\u{25C7}', None,
    // 0x28-0x2F
    None,        None,        None,        None,
    None,        None,        None,        '\u{2B17}',
    // 0x30-0x37: medium, white shapes
    '\u{25A1}', '\u{25B3}', '\u{25C1}', '\u{25BD}',
    '\u{25B7}', '\u{25CB}', '\u{2B25}', None,
    // 0x38-0x3F
    None,        None,        None,        None,
    None,        None,        None,        '\u{2B17}',
    // 0x40-0x47: small, white shapes
    '\u{25AB}', '\u{25B5}', '\u{25C3}', '\u{25BF}',
    '\u{25B9}', '\u{25CB}', '\u{2B2A}', None,
    // 0x48-0x4F
    None,        None,        None,        None,
    None,        None,        None,        '\u{2B17}',
    // 0x50-0x57: large, black shapes
    '\u{25A0}', '\u{25B2}', '\u{25C0}', '\u{25BC}',
    '\u{25B6}', '\u{25CF}', '\u{25C6}', None,
    // 0x58-0x5F
    None,        None,        None,        None,
    None,        None,        None,        '\u{2B19}',
    // 0x60-0x67: medium, black shapes
    '\u{25FC}', '\u{25B2}', '\u{25C0}', '\u{25BC}',
    '\u{25B6}', '\u{25CF}', '\u{2B26}', None,
    // 0x68-0x6F
    None,        None,        None,        None,
    None,        None,        None,        '\u{2B19}',
    // 0x70-0x77: small, black shapes
    '\u{25AA}', '\u{25B4}', '\u{25C2}', '\u{25BE}',
    '\u{25B8}', '\u{25CF}', '\u{2B2B}', None,
    // 0x78-0x7F
    None,        None,        None,        None,
    None,        None,        None,        '\u{2B19}'
  ]);
});
