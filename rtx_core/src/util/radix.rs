// /=====================================================================\ #
// |  LaTeXML::Util::Radix                                               | #
// | PostProcessing driver                                               | #
// |=====================================================================| #
// | Part of LaTeXML:                                                    | #
// |  Public domain software, produced as part of work done by the       | #
// |  United States Government & not subject to copyright in the US.     | #
// |---------------------------------------------------------------------| #
// | Bruce Miller <bruce.miller@nist.gov>                        #_//     | #
// | http://dlmf.nist.gov/LaTeXML/                              (o o)    | #
// \=========================================================ooo==U==ooo=/ #

//======================================================================
// This isn't really any sort of general purpose Radix module,
// probably the term "radix" is a misnomer here!
// It is used to primarily generate labels, or uniquifying suffixes to make ID's,
// Bibtex year tags like 2013a, etc  using alphabetic letters, or
// perhaps greek, or even from a set of symbols.
//
// The general idea is simply to generate labels in the sequence:
//   a,b,c,...y,z,aa,ab,ac,...az,ba,...zy,zz,aaa,aab,.... and so on.
// I would assume that the usual advise is that it is bad style to pass,
// or even approach "z";  However, this is an automaton, and things happen.
//======================================================================

const LETTERS: &[char] = &[
  'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];
const UP_LETTERS: &[char] = &[
  'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
const GREEK: &[char] = &[
  '\u{03B1}', '\u{03B2}', '\u{03B3}', '\u{03B4}', '\u{03B5}', '\u{03B6}', '\u{03B7}', '\u{03B8}', '\u{03B9}', '\u{03BA}', '\u{03BB}', '\u{03BC}',
  '\u{03BD}', '\u{03BE}', '\u{03BF}', '\u{03C0}', '\u{03C1}', '\u{03C3}', '\u{03C4}', '\u{03C5}', '\u{03C6}', '\u{03C7}', '\u{03C8}', '\u{03C9}',
];
const UP_GREEK: &[char] = &[
  '\u{0391}', '\u{0392}', '\u{0393}', '\u{0394}', '\u{0395}', '\u{0396}', '\u{0397}', '\u{0398}', '\u{0399}', '\u{039A}', '\u{039B}', '\u{039C}',
  '\u{039D}', '\u{039E}', '\u{039F}', '\u{03A0}', '\u{03A1}', '\u{03A3}', '\u{03A4}', '\u{03A5}', '\u{03A6}', '\u{03A7}', '\u{03A8}', '\u{03A9}',
];

pub fn radix_format(mut number: i32, symbols: &[char]) -> String {
  let mut text = String::new();
  let max = symbols.len();
  while number > 0 {
    let index = (number - 1) % (max as i32);
    text = symbols[index as usize].to_string() + &text;
    number = (number - 1) / (max as i32);
  }
  text
}
pub fn radix_format_str(mut number: i32, symbols: &[&str]) -> String {
  let mut text = String::new();
  let max = symbols.len();
  while number > 0 {
    let index = (number - 1) % (max as i32);
    text = symbols[index as usize].to_string() + &text;
    number = (number - 1) / (max as i32);
  }
  text
}

pub fn radix_alpha(n: i32) -> String { radix_format(n, &LETTERS) }

pub fn radix_up_alpha(n: i32) -> String { radix_format(n, &UP_LETTERS) }

pub fn radix_greek(n: i32) -> String { radix_format(n, &GREEK) }

pub fn radix_up_greek(n: i32) -> String { radix_format(n, &UP_GREEK) }

// Dumb place for this, but where else...
// Note: This is one 'The TeX Way'! (bah!! hint: try a large number)
// namely, it's very limited.... what happened to my much-improved version?
const RMLETTERS: &[char] = &['i', 'v', 'x', 'l', 'c', 'd', 'm']; // [CONSTANT]

pub fn radix_roman(mut n: i32) -> String {
  let mut s = String::new();
  let mut div = 1000;
  if n > div {
    s = (0..(n / div)).map(|_| 'm').collect::<String>();
  }

  let mut p = 4;
  loop {
    n %= div;
    if n == 0 {
      break;
    }
    div /= 10;
    let mut d: i32 = n / div;
    if d % 5 == 4 {
      s.push(RMLETTERS[p]);
      d += 1;
    }
    if d > 4 {
      let index: usize = p + (d / 5) as usize;
      s.push(RMLETTERS[index]);
      d %= 5;
    }
    if d != 0 {
      let ps = (0..d).map(|_| RMLETTERS[p]).collect::<String>();
      s.push_str(&ps);
    }
    if p > 1 {
      p -= 2;
    } else {
      p = 0;
    }
  }
  s
}

/// Convert the number to lower case roman numerals, returning a list of LaTeXML::Core::Token
pub fn radix_up_roman(n: i32) -> String { radix_roman(n).to_uppercase() }
