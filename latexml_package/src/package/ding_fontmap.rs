//! ding font encoding (from ding.fontmap.ltxml)
use crate::prelude::*;

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("ding", mixrc![
    // 0x00-0x07: not in unicode: left-directed scissors
    '\u{2701}', '\u{2702}', '\u{2703}', '\u{2701}', '\u{2702}', '\u{2703}', '\u{2704}', '\u{2704}',
    // 0x08-0x0F
    '\u{260E}', '\u{2706}', '\u{2707}', '\u{2708}', '\u{2709}', '\u{261B}', '\u{261A}', '\u{261B}',
    // 0x10-0x17: not in unicode: handcuff right/left-up, hand right/left-up
    '\u{261A}', '\u{1F599}', '\u{1F598}', '\u{1F599}', '\u{1F598}', '\u{270C}', '\u{270D}', '\u{270F}',
    // 0x18-0x1F
    '\u{1F589}', '\u{2710}', '\u{1F589}', '\u{270E}', '\u{1F589}', '\u{2711}', '\u{2711}', '\u{2712}',
    // 0x20-0x27: only right nib in unicode
    '\u{2712}', '\u{2713}', '\u{2714}', '\u{2715}', '\u{2716}', '\u{2717}', '\u{2719}', '\u{271A}',
    // 0x28-0x2F: no "bold outline" cross in unicode
    '\u{271C}', '\u{271B}', '\u{271D}', '\u{271E}', '\u{271F}', '\u{271F}', '\u{2720}', '\u{2736}',
    // 0x30-0x37
    '\u{2721}', '\u{2722}', '\u{2723}', '\u{2724}', '\u{2725}', '\u{2726}', '\u{2727}', '\u{26E4}',
    // 0x38-0x3F
    '\u{2605}', '\u{2606}', '\u{272A}', '\u{272B}', '\u{272C}', '\u{272D}', '\u{272E}', '\u{272F}',
    // 0x40-0x47: no asterisk thin-center-open
    '\u{2730}', '\u{2731}', '\u{2732}', '\u{2733}', '\u{2733}', '\u{2734}', '\u{2735}', '\u{2736}',
    // 0x48-0x4F
    '\u{2737}', '\u{2738}', '\u{2739}', '\u{273A}', '\u{273B}', '\u{273C}', '\u{273D}', '\u{273E}',
    // 0x50-0x57: \SixFlowerPetalDotted not in unicode; \FourClowerOpen,\FourClowerSolid not in unicode
    '\u{273F}', '\u{273E}', '\u{2740}', '\u{2741}', '\u{2742}', '\u{2743}', '\u{1F340}', '\u{1F340}',
    // 0x58-0x5F: \SixFlowerRemovedOpenPetal not in unicode
    '\u{2749}', '\u{274A}', '\u{274B}', '\u{273E}', '\u{2748}', '\u{2747}', '\u{2744}', '\u{2746}',
    // 0x60-0x67: \EllipseShadow not in unicode
    '\u{2745}', '\u{25CF}', '\u{2B2D}', '\u{2B2C}', '\u{274D}', '\u{2B2D}', '\u{25A1}', '\u{25A0}',
    // 0x68-0x6F: \SquareShadowTopLeft not in unicode; \SquareCastShadowTopLeft not in unicode
    '\u{274F}', '\u{2750}', '\u{2750}', '\u{2751}', '\u{2752}', '\u{2750}', '\u{25B2}', '\u{25BC}',
    // 0x70-0x77
    '\u{25C6}', '\u{2756}', '\u{25D7}', '\u{25D6}', '\u{2758}', '\u{2759}', '\u{275A}', '\u{279F}',
    // 0x78-0x7B
    '\u{27A6}', '\u{27A5}', '\u{27A7}', '\u{27B2}'
  ]);
});
