//! LGR font encoding (from lgr.fontmap.ltxml)
//! Greek font encoding with ligatures for polytonic accents.
//!
//! The slots follow the glyphs the LGR fonts carry: the CB fonts' encoding
//! vector (fonts/enc/dvips/cbfonts/CB.enc, glyph names read through the Adobe
//! Glyph List) and lgrenc.def's `\DeclareTextSymbol`s. Perl's map differs at 12
//! slots, where it names a symbol variant or a different letter (`U` → ϒ, `j` →
//! ϑ, `k` → ϰ, `\"U` → ϔ, …; the pdflatex PDFs of teubner-doc, test-lgrenc and
//! char-list extract Υ, θ, κ, Ϋ there), and leaves 7 of the font's glyphs
//! unmapped (KPE); the breathings, the tonos and the ypogegrammeni (0x3C, 0x3E,
//! 0x27, 0x7C) are the Greek spacing marks, not quotation marks.
use crate::prelude::*;

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("LGR", mixrc![
    // 0x00-0x07 — 2-5 are the Attic acrophonic numerals (lgrenc.def
    // `\textpentedeka` …), 6 the small stigma ϛ (`\textstigma`, CB.enc uni03DB).
    '\u{2013}', None,       '\u{10144}','\u{10145}','\u{10146}','\u{10147}','\u{03DB}', '\u{03DB}',
    // 0x08-0x0F — 13 and 15 are variant Ϋ and ϋ (CB.enc uni03AB.alt, uni03CB.alt).
    '\u{1FBE}', '\u{1FBC}', '\u{1FCC}', '\u{1FFC}', '\u{0391}', '\u{03AB}', '\u{03B1}', '\u{03CB}',
    // 0x10-0x17 — 18 koppa ϟ (`\textkoppa`), 21 archaic Koppa Ϙ (`\textQoppa`),
    // 22 Stigma Ϛ (`\textStigma`).
    '\u{02CF}', '\u{02CE}', '\u{03DF}', '\u{03D9}', None,       '\u{03D8}', '\u{03DA}', '\u{03E0}',
    // 0x18-0x1F — 26 is the small schwa ə (`\textschwa`).
    '\u{20AC}', '\u{2030}', '\u{0259}', '\u{03E1}', '\u{2018}', '\u{2019}', '\u{02D8}', '\u{00AF}',
    // 0x20-0x27 — slot 0x22 `"` is the dialytika (lgrenc.def:434
    // `\DeclareTextAccent{\accdialytika}{LGR}{34}`), not the psili Perl's map
    // put there (KPE #148); 0x26 is the middle dot (CB.enc periodcentered),
    // 0x27 `'` the tonos (CB.enc tonos).
    '\u{1FC1}', '!',        '\u{00A8}', '\u{1FEE}', '\u{1FED}', '%',        '\u{00B7}', '\u{0384}',
    // 0x28-0x2F
    '(',        ')',        '*',        '+',        ',',        '-',        '.',        '/',
    // 0x30-0x37
    '0',        '1',        '2',        '3',        '4',        '5',        '6',        '7',
    // 0x38-0x3F — `<` and `>` are the dasia ῾ and psili ᾿ (CB.enc uni1FFE,
    // uni1FBF; the babel test PDF's `>En` is ᾿Εν), not the quotation marks
    // ‛ ’, which also left every ’ before a vowel in any text a ligature key.
    '8',        '9',        ':',        '\u{0387}', '\u{1FFE}', '=',        '\u{1FBF}', ';',

    // 0x40-0x47
    '\u{1FDF}', '\u{0391}', '\u{0392}', '\u{1FDD}', '\u{0394}', '\u{0395}', '\u{03A6}', '\u{0393}',
    // 0x48-0x4F
    '\u{0397}', '\u{0399}', '\u{0398}', '\u{039A}', '\u{039B}', '\u{039C}', '\u{039D}', '\u{039F}',
    // 0x50-0x57 — 85 `U` is Upsilon Υ (lgrenc.def:167, CB.enc /Upsilon), not
    // the upsilon-with-hook symbol ϒ.
    '\u{03A0}', '\u{03A7}', '\u{03A1}', '\u{03A3}', '\u{03A4}', '\u{03A5}', '\u{1FDE}', '\u{03A9}',
    // 0x58-0x5F
    '\u{039E}', '\u{03A8}', '\u{0396}', '[',        '\u{1FCF}', ']',        '\u{1FCE}', '\u{1FCD}',
    // 0x60-0x67
    '\u{1FEF}', '\u{03B1}', '\u{03B2}', '\u{03C2}', '\u{03B4}', '\u{03B5}', '\u{03C6}', '\u{03B3}',
    // 0x68-0x6F — `j` θ and `k` κ (lgrenc.def `\texttheta`/`\textkappa`
    // 106/107, CB.enc /theta /kappa), not the symbols ϑ ϰ.
    '\u{03B7}', '\u{03B9}', '\u{03B8}', '\u{03BA}', '\u{03BB}', '\u{03BC}', '\u{03BD}', '\u{03BF}',
    // 0x70-0x77 — slot 0x73 `s` is σ (lgrenc.def:190-192 `\textsigma` =
    // `s\noboundary`, final ς is slot 0x63 `c`; the LGR font's word-end
    // ligature to ς is not modelled). Perl's map put ς here: KPE #148.
    '\u{03C0}', '\u{03C7}', '\u{03C1}', '\u{03C3}', '\u{03C4}', '\u{03C5}', None,       '\u{03C9}',
    // 0x78-0x7F — 124 `|` is the ypogegrammeni ͺ (lgrenc.def:450, CB.enc uni037A).
    '\u{03BE}', '\u{03C8}', '\u{03B6}', '\u{00AB}', '\u{037A}', '\u{00BB}', '\u{1FC0}', '\u{2014}',

    // 0x80-0x87
    '\u{1F70}', '\u{1F01}', '\u{1F00}', '\u{1F03}', '\u{1FB2}', '\u{1F81}', '\u{1F80}', '\u{1F83}',
    // 0x88-0x8F
    '\u{1F71}', '\u{1F05}', '\u{1F04}', '\u{1F02}', '\u{1FB4}', '\u{1F85}', '\u{1F84}', '\u{1F82}',
    // 0x90-0x97
    '\u{1FB6}', '\u{1F07}', '\u{1F06}', '\u{03DD}', '\u{1FB7}', '\u{1F87}', '\u{1F86}', None,
    // 0x98-0x9F
    '\u{1F74}', '\u{1F21}', '\u{1F20}', None,       '\u{1FC2}', '\u{1F91}', '\u{1F90}', None,
    // 0xA0-0xA7
    '\u{1F75}', '\u{1F25}', '\u{1F24}', '\u{1F23}', '\u{1FC4}', '\u{1F95}', '\u{1F94}', '\u{1F93}',
    // 0xA8-0xAF
    '\u{1FC6}', '\u{1F27}', '\u{1F26}', '\u{1F22}', '\u{1FC7}', '\u{1F97}', '\u{1F96}', '\u{1F92}',
    // 0xB0-0xB7
    '\u{1F7C}', '\u{1F61}', '\u{1F60}', '\u{1F63}', '\u{1FF2}', '\u{1FA1}', '\u{1FA0}', '\u{1FA3}',
    // 0xB8-0xBF — 184 is ώ (lgrenc.def `\acctonos` w, CB.enc omegatonos), the
    // oxia form like its siblings; 0xB0 is ὼ.
    '\u{1F7D}', '\u{1F65}', '\u{1F64}', '\u{1F62}', '\u{1FF4}', '\u{1FA5}', '\u{1FA4}', '\u{1FA2}',

    // 0xC0-0xC7
    '\u{1FF6}', '\u{1F67}', '\u{1F66}', '\u{03DC}', '\u{1FF7}', '\u{1FA7}', '\u{1FA6}', None,
    // 0xC8-0xCF
    '\u{1F76}', '\u{1F31}', '\u{1F30}', '\u{1F33}', '\u{1F7A}', '\u{1F51}', '\u{1F50}', '\u{1F53}',
    // 0xD0-0xD7
    '\u{1F77}', '\u{1F35}', '\u{1F34}', '\u{1F32}', '\u{1F7B}', '\u{1F55}', '\u{1F54}', '\u{1F52}',
    // 0xD8-0xDF — 223 is Ϋ (lgrenc.def:861 `\accdialytika` U, CB.enc
    // Upsilondieresis): the uppercase hiatus ΑΫΛΟΣ (teubner-doc).
    '\u{1FD6}', '\u{1F37}', '\u{1F36}', '\u{03AA}', '\u{1FE6}', '\u{1F57}', '\u{1F56}', '\u{03AB}',
    // 0xE0-0xE7
    '\u{1F72}', '\u{1F11}', '\u{1F10}', '\u{1F13}', '\u{1F78}', '\u{1F41}', '\u{1F40}', '\u{1F43}',
    // 0xE8-0xEF
    '\u{1F73}', '\u{1F15}', '\u{1F14}', '\u{1F12}', '\u{1F79}', '\u{1F45}', '\u{1F44}', '\u{1F42}',
    // 0xF0-0xF7
    '\u{03CA}', '\u{1FD2}', '\u{1FD3}', '\u{1FD7}', '\u{03CB}', '\u{1FE2}', '\u{1FE3}', '\u{1FE7}',
    // 0xF8-0xFF
    '\u{1FB3}', '\u{1FC3}', '\u{1FF3}', '\u{1FE5}', '\u{1FE4}', None,       '\u{0374}', '\u{0375}'
  ]);

  // The CB fonts' iota ligatures (grmn1000.tfm LIGTABLE: a vowel slot followed
  // by `|`, O 174, is its iota-subscript slot — `(LABEL O 202) (LIG O 174 O
  // 206)`, ἀ → ᾀ; the capitals `(LABEL C A) (LIG O 174 O 11)`, Α → ᾼ). The
  // accent table below composes only the separate marks (`>a|`), and a kernel
  // accent's declared composite prints the precomposed vowel itself
  // (`\accpsili\textalpha` is slot 130 ἀ, lgrenc.def:615), so UTF-8 ᾀ
  // (lgrenc.dfu `\ensuregreek{\accpsili\textalpha\ypogegrammeni}`) came out ἀͺ
  // and ῼ Ωͺ where pdflatex prints ᾀ and ῼ (greek-fontenc hyperref-with-greek,
  // char-list). Perl has no composites and no such ligatures (KNOWN_PERL_ERRORS
  // #290, OXIDIZED_DESIGN_DIVERGENCES #312). α η ω are in the table below. One
  // pass for the 36 vowels, defined first, so it applies after every accent
  // ligature has composed its vowel.
  const IOTA_FORMS: [(char, char); 36] = [
    ('\u{0391}', '\u{1FBC}'),
    ('\u{0397}', '\u{1FCC}'),
    ('\u{03A9}', '\u{1FFC}'),
    ('\u{1F70}', '\u{1FB2}'),
    ('\u{1F01}', '\u{1F81}'),
    ('\u{1F00}', '\u{1F80}'),
    ('\u{1F03}', '\u{1F83}'),
    ('\u{1F71}', '\u{1FB4}'),
    ('\u{1F05}', '\u{1F85}'),
    ('\u{1F04}', '\u{1F84}'),
    ('\u{1F02}', '\u{1F82}'),
    ('\u{1FB6}', '\u{1FB7}'),
    ('\u{1F07}', '\u{1F87}'),
    ('\u{1F06}', '\u{1F86}'),
    ('\u{1F74}', '\u{1FC2}'),
    ('\u{1F21}', '\u{1F91}'),
    ('\u{1F20}', '\u{1F90}'),
    ('\u{1F75}', '\u{1FC4}'),
    ('\u{1F25}', '\u{1F95}'),
    ('\u{1F24}', '\u{1F94}'),
    ('\u{1F23}', '\u{1F93}'),
    ('\u{1FC6}', '\u{1FC7}'),
    ('\u{1F27}', '\u{1F97}'),
    ('\u{1F26}', '\u{1F96}'),
    ('\u{1F22}', '\u{1F92}'),
    ('\u{1F7C}', '\u{1FF2}'),
    ('\u{1F61}', '\u{1FA1}'),
    ('\u{1F60}', '\u{1FA0}'),
    ('\u{1F63}', '\u{1FA3}'),
    ('\u{1F7D}', '\u{1FF4}'),
    ('\u{1F65}', '\u{1FA5}'),
    ('\u{1F64}', '\u{1FA4}'),
    ('\u{1F62}', '\u{1FA2}'),
    ('\u{1FF6}', '\u{1FF7}'),
    ('\u{1F67}', '\u{1FA7}'),
    ('\u{1F66}', '\u{1FA6}'),
  ];
  let vowels: String = IOTA_FORMS.iter().map(|(vowel, _)| *vowel).collect();
  DefLigature!(
    &format!("([{vowels}])\u{037A}"),
    |caps: &regex::Captures| {
      let vowel = caps[1].chars().next();
      IOTA_FORMS
        .iter()
        .find(|(form_of, _)| Some(*form_of) == vowel)
        .map_or_else(
          || caps[0].to_string(),
          |(_, with_iota)| with_iota.to_string(),
        )
    }
  );

  // Greek polytonic accent ligatures.
  // These map sequences of LGR-encoded characters (accents + base letters) to precomposed forms.
  // Generated from the Perl ligature computation in lgr.fontmap.ltxml, over the
  // map above, from its accent table with one correction: Perl lists the
  // upsilon and omicron slots 0xCC-0xCF, 0xD4-0xD7, 0xDC-0xDE, 0xE4-0xE7 and
  // 0xEC-0xEF as `` `u| ``, `<o|` …, with an iota subscript neither vowel takes
  // (CB.enc: uni1F7A ὺ, uni1F41 ὁ …), so `l'ogos` kept a bare `´ο` where
  // pdflatex prints λόγος (KPE).
  // Sorted by length then lexicographically, matching Perl's sort order.

  // 2-char ligatures
  DefLigature!("\u{00A8}\u{0384}", "\u{1FEE}");
  DefLigature!("\u{00A8}\u{0399}", "\u{03AA}");
  DefLigature!("\u{00A8}\u{03A5}", "\u{03AB}");
  DefLigature!("\u{00A8}\u{03B9}", "\u{03CA}");
  DefLigature!("\u{00A8}\u{03C5}", "\u{03CB}");
  DefLigature!("\u{00A8}\u{1FC0}", "\u{1FC1}");
  DefLigature!("\u{00A8}\u{1FEF}", "\u{1FED}");
  DefLigature!("\u{0384}\u{03B1}", "\u{1F71}");
  DefLigature!("\u{0384}\u{03B5}", "\u{1F73}");
  DefLigature!("\u{0384}\u{03B7}", "\u{1F75}");
  DefLigature!("\u{0384}\u{03B9}", "\u{1F77}");
  DefLigature!("\u{0384}\u{03BF}", "\u{1F79}");
  DefLigature!("\u{0384}\u{03C5}", "\u{1F7B}");
  DefLigature!("\u{0384}\u{03C9}", "\u{1F7D}");
  DefLigature!("\u{03B1}\u{037A}", "\u{1FB3}");
  DefLigature!("\u{03B7}\u{037A}", "\u{1FC3}");
  DefLigature!("\u{03C9}\u{037A}", "\u{1FF3}");
  DefLigature!("\u{1FBF}\u{0384}", "\u{1FCE}");
  DefLigature!("\u{1FBF}\u{03B1}", "\u{1F00}");
  DefLigature!("\u{1FBF}\u{03B5}", "\u{1F10}");
  DefLigature!("\u{1FBF}\u{03B7}", "\u{1F20}");
  DefLigature!("\u{1FBF}\u{03B9}", "\u{1F30}");
  DefLigature!("\u{1FBF}\u{03BF}", "\u{1F40}");
  DefLigature!("\u{1FBF}\u{03C1}", "\u{1FE4}");
  DefLigature!("\u{1FBF}\u{03C5}", "\u{1F50}");
  DefLigature!("\u{1FBF}\u{03C9}", "\u{1F60}");
  DefLigature!("\u{1FBF}\u{1FC0}", "\u{1FCF}");
  DefLigature!("\u{1FBF}\u{1FEF}", "\u{1FCD}");
  DefLigature!("\u{1FC0}\u{03B1}", "\u{1FB6}");
  DefLigature!("\u{1FC0}\u{03B7}", "\u{1FC6}");
  DefLigature!("\u{1FC0}\u{03B9}", "\u{1FD6}");
  DefLigature!("\u{1FC0}\u{03C5}", "\u{1FE6}");
  DefLigature!("\u{1FC0}\u{03C9}", "\u{1FF6}");
  DefLigature!("\u{1FEF}\u{03B1}", "\u{1F70}");
  DefLigature!("\u{1FEF}\u{03B5}", "\u{1F72}");
  DefLigature!("\u{1FEF}\u{03B7}", "\u{1F74}");
  DefLigature!("\u{1FEF}\u{03B9}", "\u{1F76}");
  DefLigature!("\u{1FEF}\u{03BF}", "\u{1F78}");
  DefLigature!("\u{1FEF}\u{03C5}", "\u{1F7A}");
  DefLigature!("\u{1FEF}\u{03C9}", "\u{1F7C}");
  DefLigature!("\u{1FFE}\u{0384}", "\u{1FDE}");
  DefLigature!("\u{1FFE}\u{03B1}", "\u{1F01}");
  DefLigature!("\u{1FFE}\u{03B5}", "\u{1F11}");
  DefLigature!("\u{1FFE}\u{03B7}", "\u{1F21}");
  DefLigature!("\u{1FFE}\u{03B9}", "\u{1F31}");
  DefLigature!("\u{1FFE}\u{03BF}", "\u{1F41}");
  DefLigature!("\u{1FFE}\u{03C1}", "\u{1FE5}");
  DefLigature!("\u{1FFE}\u{03C5}", "\u{1F51}");
  DefLigature!("\u{1FFE}\u{03C9}", "\u{1F61}");
  DefLigature!("\u{1FFE}\u{1FC0}", "\u{1FDF}");
  DefLigature!("\u{1FFE}\u{1FEF}", "\u{1FDD}");

  // 3-char ligatures
  DefLigature!("\u{00A8}\u{0384}\u{03B9}", "\u{1FD3}");
  DefLigature!("\u{00A8}\u{0384}\u{03C5}", "\u{1FE3}");
  DefLigature!("\u{00A8}\u{1FC0}\u{03B9}", "\u{1FD7}");
  DefLigature!("\u{00A8}\u{1FC0}\u{03C5}", "\u{1FE7}");
  DefLigature!("\u{00A8}\u{1FEF}\u{03B9}", "\u{1FD2}");
  DefLigature!("\u{00A8}\u{1FEF}\u{03C5}", "\u{1FE2}");
  DefLigature!("\u{0384}\u{00A8}\u{03B9}", "\u{1FD3}");
  DefLigature!("\u{0384}\u{00A8}\u{03C5}", "\u{1FE3}");
  DefLigature!("\u{0384}\u{03B1}\u{037A}", "\u{1FB4}");
  DefLigature!("\u{0384}\u{03B7}\u{037A}", "\u{1FC4}");
  DefLigature!("\u{0384}\u{03C9}\u{037A}", "\u{1FF4}");
  DefLigature!("\u{0384}\u{1FBF}\u{03B1}", "\u{1F04}");
  DefLigature!("\u{0384}\u{1FBF}\u{03B5}", "\u{1F14}");
  DefLigature!("\u{0384}\u{1FBF}\u{03B7}", "\u{1F24}");
  DefLigature!("\u{0384}\u{1FBF}\u{03B9}", "\u{1F34}");
  DefLigature!("\u{0384}\u{1FBF}\u{03BF}", "\u{1F44}");
  DefLigature!("\u{0384}\u{1FBF}\u{03C5}", "\u{1F54}");
  DefLigature!("\u{0384}\u{1FBF}\u{03C9}", "\u{1F64}");
  DefLigature!("\u{0384}\u{1FFE}\u{03B1}", "\u{1F05}");
  DefLigature!("\u{0384}\u{1FFE}\u{03B5}", "\u{1F15}");
  DefLigature!("\u{0384}\u{1FFE}\u{03B7}", "\u{1F25}");
  DefLigature!("\u{0384}\u{1FFE}\u{03B9}", "\u{1F35}");
  DefLigature!("\u{0384}\u{1FFE}\u{03BF}", "\u{1F45}");
  DefLigature!("\u{0384}\u{1FFE}\u{03C5}", "\u{1F55}");
  DefLigature!("\u{0384}\u{1FFE}\u{03C9}", "\u{1F65}");
  DefLigature!("\u{1FBF}\u{0384}\u{03B1}", "\u{1F04}");
  DefLigature!("\u{1FBF}\u{0384}\u{03B5}", "\u{1F14}");
  DefLigature!("\u{1FBF}\u{0384}\u{03B7}", "\u{1F24}");
  DefLigature!("\u{1FBF}\u{0384}\u{03B9}", "\u{1F34}");
  DefLigature!("\u{1FBF}\u{0384}\u{03BF}", "\u{1F44}");
  DefLigature!("\u{1FBF}\u{0384}\u{03C5}", "\u{1F54}");
  DefLigature!("\u{1FBF}\u{0384}\u{03C9}", "\u{1F64}");
  DefLigature!("\u{1FBF}\u{03B1}\u{037A}", "\u{1F80}");
  DefLigature!("\u{1FBF}\u{03B7}\u{037A}", "\u{1F90}");
  DefLigature!("\u{1FBF}\u{03C9}\u{037A}", "\u{1FA0}");
  DefLigature!("\u{1FBF}\u{1FC0}\u{03B1}", "\u{1F06}");
  DefLigature!("\u{1FBF}\u{1FC0}\u{03B7}", "\u{1F26}");
  DefLigature!("\u{1FBF}\u{1FC0}\u{03B9}", "\u{1F36}");
  DefLigature!("\u{1FBF}\u{1FC0}\u{03C5}", "\u{1F56}");
  DefLigature!("\u{1FBF}\u{1FC0}\u{03C9}", "\u{1F66}");
  DefLigature!("\u{1FBF}\u{1FEF}\u{03B1}", "\u{1F02}");
  DefLigature!("\u{1FBF}\u{1FEF}\u{03B5}", "\u{1F12}");
  DefLigature!("\u{1FBF}\u{1FEF}\u{03B7}", "\u{1F22}");
  DefLigature!("\u{1FBF}\u{1FEF}\u{03B9}", "\u{1F32}");
  DefLigature!("\u{1FBF}\u{1FEF}\u{03BF}", "\u{1F42}");
  DefLigature!("\u{1FBF}\u{1FEF}\u{03C5}", "\u{1F52}");
  DefLigature!("\u{1FBF}\u{1FEF}\u{03C9}", "\u{1F62}");
  DefLigature!("\u{1FC0}\u{00A8}\u{03B9}", "\u{1FD7}");
  DefLigature!("\u{1FC0}\u{00A8}\u{03C5}", "\u{1FE7}");
  DefLigature!("\u{1FC0}\u{03B1}\u{037A}", "\u{1FB7}");
  DefLigature!("\u{1FC0}\u{03B7}\u{037A}", "\u{1FC7}");
  DefLigature!("\u{1FC0}\u{03C9}\u{037A}", "\u{1FF7}");
  DefLigature!("\u{1FC0}\u{1FBF}\u{03B1}", "\u{1F06}");
  DefLigature!("\u{1FC0}\u{1FBF}\u{03B7}", "\u{1F26}");
  DefLigature!("\u{1FC0}\u{1FBF}\u{03B9}", "\u{1F36}");
  DefLigature!("\u{1FC0}\u{1FBF}\u{03C5}", "\u{1F56}");
  DefLigature!("\u{1FC0}\u{1FBF}\u{03C9}", "\u{1F66}");
  DefLigature!("\u{1FC0}\u{1FFE}\u{03B1}", "\u{1F07}");
  DefLigature!("\u{1FC0}\u{1FFE}\u{03B7}", "\u{1F27}");
  DefLigature!("\u{1FC0}\u{1FFE}\u{03B9}", "\u{1F37}");
  DefLigature!("\u{1FC0}\u{1FFE}\u{03C5}", "\u{1F57}");
  DefLigature!("\u{1FC0}\u{1FFE}\u{03C9}", "\u{1F67}");
  DefLigature!("\u{1FEF}\u{00A8}\u{03B9}", "\u{1FD2}");
  DefLigature!("\u{1FEF}\u{00A8}\u{03C5}", "\u{1FE2}");
  DefLigature!("\u{1FEF}\u{03B1}\u{037A}", "\u{1FB2}");
  DefLigature!("\u{1FEF}\u{03B7}\u{037A}", "\u{1FC2}");
  DefLigature!("\u{1FEF}\u{03C9}\u{037A}", "\u{1FF2}");
  DefLigature!("\u{1FEF}\u{1FBF}\u{03B1}", "\u{1F02}");
  DefLigature!("\u{1FEF}\u{1FBF}\u{03B5}", "\u{1F12}");
  DefLigature!("\u{1FEF}\u{1FBF}\u{03B7}", "\u{1F22}");
  DefLigature!("\u{1FEF}\u{1FBF}\u{03B9}", "\u{1F32}");
  DefLigature!("\u{1FEF}\u{1FBF}\u{03BF}", "\u{1F42}");
  DefLigature!("\u{1FEF}\u{1FBF}\u{03C5}", "\u{1F52}");
  DefLigature!("\u{1FEF}\u{1FBF}\u{03C9}", "\u{1F62}");
  DefLigature!("\u{1FEF}\u{1FFE}\u{03B1}", "\u{1F03}");
  DefLigature!("\u{1FEF}\u{1FFE}\u{03B5}", "\u{1F13}");
  DefLigature!("\u{1FEF}\u{1FFE}\u{03B7}", "\u{1F23}");
  DefLigature!("\u{1FEF}\u{1FFE}\u{03B9}", "\u{1F33}");
  DefLigature!("\u{1FEF}\u{1FFE}\u{03BF}", "\u{1F43}");
  DefLigature!("\u{1FEF}\u{1FFE}\u{03C5}", "\u{1F53}");
  DefLigature!("\u{1FEF}\u{1FFE}\u{03C9}", "\u{1F63}");
  DefLigature!("\u{1FFE}\u{0384}\u{03B1}", "\u{1F05}");
  DefLigature!("\u{1FFE}\u{0384}\u{03B5}", "\u{1F15}");
  DefLigature!("\u{1FFE}\u{0384}\u{03B7}", "\u{1F25}");
  DefLigature!("\u{1FFE}\u{0384}\u{03B9}", "\u{1F35}");
  DefLigature!("\u{1FFE}\u{0384}\u{03BF}", "\u{1F45}");
  DefLigature!("\u{1FFE}\u{0384}\u{03C5}", "\u{1F55}");
  DefLigature!("\u{1FFE}\u{0384}\u{03C9}", "\u{1F65}");
  DefLigature!("\u{1FFE}\u{03B1}\u{037A}", "\u{1F81}");
  DefLigature!("\u{1FFE}\u{03B7}\u{037A}", "\u{1F91}");
  DefLigature!("\u{1FFE}\u{03C9}\u{037A}", "\u{1FA1}");
  DefLigature!("\u{1FFE}\u{1FC0}\u{03B1}", "\u{1F07}");
  DefLigature!("\u{1FFE}\u{1FC0}\u{03B7}", "\u{1F27}");
  DefLigature!("\u{1FFE}\u{1FC0}\u{03B9}", "\u{1F37}");
  DefLigature!("\u{1FFE}\u{1FC0}\u{03C5}", "\u{1F57}");
  DefLigature!("\u{1FFE}\u{1FC0}\u{03C9}", "\u{1F67}");
  DefLigature!("\u{1FFE}\u{1FEF}\u{03B1}", "\u{1F03}");
  DefLigature!("\u{1FFE}\u{1FEF}\u{03B5}", "\u{1F13}");
  DefLigature!("\u{1FFE}\u{1FEF}\u{03B7}", "\u{1F23}");
  DefLigature!("\u{1FFE}\u{1FEF}\u{03B9}", "\u{1F33}");
  DefLigature!("\u{1FFE}\u{1FEF}\u{03BF}", "\u{1F43}");
  DefLigature!("\u{1FFE}\u{1FEF}\u{03C5}", "\u{1F53}");
  DefLigature!("\u{1FFE}\u{1FEF}\u{03C9}", "\u{1F63}");

  // 4-char ligatures
  DefLigature!("\u{0384}\u{1FBF}\u{03B1}\u{037A}", "\u{1F84}");
  DefLigature!("\u{0384}\u{1FBF}\u{03B7}\u{037A}", "\u{1F94}");
  DefLigature!("\u{0384}\u{1FBF}\u{03C9}\u{037A}", "\u{1FA4}");
  DefLigature!("\u{0384}\u{1FFE}\u{03B1}\u{037A}", "\u{1F85}");
  DefLigature!("\u{0384}\u{1FFE}\u{03B7}\u{037A}", "\u{1F95}");
  DefLigature!("\u{0384}\u{1FFE}\u{03C9}\u{037A}", "\u{1FA5}");
  DefLigature!("\u{1FBF}\u{0384}\u{03B1}\u{037A}", "\u{1F84}");
  DefLigature!("\u{1FBF}\u{0384}\u{03B7}\u{037A}", "\u{1F94}");
  DefLigature!("\u{1FBF}\u{0384}\u{03C9}\u{037A}", "\u{1FA4}");
  DefLigature!("\u{1FBF}\u{1FC0}\u{03B1}\u{037A}", "\u{1F86}");
  DefLigature!("\u{1FBF}\u{1FC0}\u{03B7}\u{037A}", "\u{1F96}");
  DefLigature!("\u{1FBF}\u{1FC0}\u{03C9}\u{037A}", "\u{1FA6}");
  DefLigature!("\u{1FBF}\u{1FEF}\u{03B1}\u{037A}", "\u{1F82}");
  DefLigature!("\u{1FBF}\u{1FEF}\u{03B7}\u{037A}", "\u{1F92}");
  DefLigature!("\u{1FBF}\u{1FEF}\u{03C9}\u{037A}", "\u{1FA2}");
  DefLigature!("\u{1FC0}\u{1FBF}\u{03B1}\u{037A}", "\u{1F86}");
  DefLigature!("\u{1FC0}\u{1FBF}\u{03B7}\u{037A}", "\u{1F96}");
  DefLigature!("\u{1FC0}\u{1FBF}\u{03C9}\u{037A}", "\u{1FA6}");
  DefLigature!("\u{1FC0}\u{1FFE}\u{03B1}\u{037A}", "\u{1F87}");
  DefLigature!("\u{1FC0}\u{1FFE}\u{03B7}\u{037A}", "\u{1F97}");
  DefLigature!("\u{1FC0}\u{1FFE}\u{03C9}\u{037A}", "\u{1FA7}");
  DefLigature!("\u{1FEF}\u{1FBF}\u{03B1}\u{037A}", "\u{1F82}");
  DefLigature!("\u{1FEF}\u{1FBF}\u{03B7}\u{037A}", "\u{1F92}");
  DefLigature!("\u{1FEF}\u{1FBF}\u{03C9}\u{037A}", "\u{1FA2}");
  DefLigature!("\u{1FEF}\u{1FFE}\u{03B1}\u{037A}", "\u{1F83}");
  DefLigature!("\u{1FEF}\u{1FFE}\u{03B7}\u{037A}", "\u{1F93}");
  DefLigature!("\u{1FEF}\u{1FFE}\u{03C9}\u{037A}", "\u{1FA3}");
  DefLigature!("\u{1FFE}\u{0384}\u{03B1}\u{037A}", "\u{1F85}");
  DefLigature!("\u{1FFE}\u{0384}\u{03B7}\u{037A}", "\u{1F95}");
  DefLigature!("\u{1FFE}\u{0384}\u{03C9}\u{037A}", "\u{1FA5}");
  DefLigature!("\u{1FFE}\u{1FC0}\u{03B1}\u{037A}", "\u{1F87}");
  DefLigature!("\u{1FFE}\u{1FC0}\u{03B7}\u{037A}", "\u{1F97}");
  DefLigature!("\u{1FFE}\u{1FC0}\u{03C9}\u{037A}", "\u{1FA7}");
  DefLigature!("\u{1FFE}\u{1FEF}\u{03B1}\u{037A}", "\u{1F83}");
  DefLigature!("\u{1FFE}\u{1FEF}\u{03B7}\u{037A}", "\u{1F93}");
  DefLigature!("\u{1FFE}\u{1FEF}\u{03C9}\u{037A}", "\u{1FA3}");
});
