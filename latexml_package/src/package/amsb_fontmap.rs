//! AMSb font encoding (from amsb.fontmap.ltxml)
//! Perl: uppercase_mathstyle => { family => 'blackboard' }
use crate::prelude::*;

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("AMSb", mixrc![
    // 0x00-0x07
    '\u{2268}', '\u{2269}', '\u{2270}', '\u{2271}', '\u{226E}', '\u{226F}', '\u{2280}', '\u{2281}',
    // 0x08-0x0F: Perl has multi-char entries at 0x0A,0x0B ("\x{2A7D}\x{0338}", "\x{2A7E}\x{0338}")
    // Rust fontmap is Option<char>, so these are None (no single-char equivalent)
    '\u{2268}', '\u{2269}', None, None, '\u{2A87}', '\u{2A88}', '\u{22E0}', '\u{22E1}',
    // 0x10-0x17: Perl has multi-char at 0x14,0x15 ("\x{2266}\x{0338}", "\x{2267}\x{0338}")
    '\u{22E8}', '\u{22E9}', '\u{22E6}', '\u{22E7}', None, None, '\u{2AB5}', '\u{2AB6}',
    // 0x18-0x1F
    '\u{2AB9}', '\u{2ABA}', '\u{2A89}', '\u{2A8A}', '\u{2241}', '\u{2247}', '\u{2571}', '\u{2572}',
    // 0x20-0x27: Perl has multi-char at 0x22,0x23 ("\x{2AC5}\x{0338}", "\x{2AC6}\x{0338}")
    '\u{228A}', '\u{228B}', None, None, '\u{2ACB}', '\u{2ACC}', '\u{2ACB}', '\u{2ACC}',
    // 0x28-0x2F
    '\u{228A}', '\u{228B}', '\u{2288}', '\u{2289}', '\u{2226}', '\u{2224}', '\u{2224}', '\u{2226}',
    // 0x30-0x37
    '\u{22AC}', '\u{22AE}', '\u{22AD}', '\u{22AF}', '\u{22ED}', '\u{22EC}', '\u{22EB}', '\u{22EA}',
    // 0x38-0x3F
    '\u{219A}', '\u{219B}', '\u{21CD}', '\u{21CF}', '\u{21CE}', '\u{21AE}', '\u{22C7}', '\u{2205}',
    // 0x40-0x47: blackboard bold letters
    '\u{2204}', '\u{1D538}', '\u{1D539}', '\u{2102}', '\u{1D53B}', '\u{1D53C}', '\u{1D53D}', '\u{1D53E}',
    // 0x48-0x4F
    '\u{210D}', '\u{1D540}', '\u{1D541}', '\u{1D542}', '\u{1D543}', '\u{1D544}', '\u{2115}', '\u{1D546}',
    // 0x50-0x57
    '\u{2119}', '\u{211A}', '\u{211D}', '\u{1D54A}', '\u{1D54B}', '\u{1D54C}', '\u{1D54D}', '\u{1D54E}',
    // 0x58-0x5F
    '\u{1D54F}', '\u{1D550}', '\u{2124}', '\u{005E}', '\u{005E}', '\u{007E}', '\u{007E}', None,
    // 0x60-0x67
    '\u{2132}', '\u{2141}', None, None, None, None, '\u{2127}', '\u{00F0}',
    // 0x68-0x6F
    '\u{2242}', '\u{2136}', '\u{2137}', '\u{2138}', '\u{22D6}', '\u{22D7}', '\u{22C9}', '\u{22CA}',
    // 0x70-0x77
    '\u{2223}', '\u{2225}', '\u{2216}', '\u{223C}', '\u{2248}', '\u{224A}', '\u{2AB8}', '\u{2AB7}',
    // 0x78-0x7F
    '\u{21B6}', '\u{21B7}', '\u{03DD}', '\u{03F0}', '\u{1D55C}', '\u{210F}', '\u{210F}', '\u{03F6}'
  ]);
});
