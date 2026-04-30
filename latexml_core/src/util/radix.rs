//! simple radix conversion utilities
//!
//! This module provides some simple utilities for radix conversion.
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
  'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
  't', 'u', 'v', 'w', 'x', 'y', 'z',
];
const UP_LETTERS: &[char] = &[
  'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
  'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
const GREEK: &[char] = &[
  '\u{03B1}', '\u{03B2}', '\u{03B3}', '\u{03B4}', '\u{03B5}', '\u{03B6}', '\u{03B7}', '\u{03B8}',
  '\u{03B9}', '\u{03BA}', '\u{03BB}', '\u{03BC}', '\u{03BD}', '\u{03BE}', '\u{03BF}', '\u{03C0}',
  '\u{03C1}', '\u{03C3}', '\u{03C4}', '\u{03C5}', '\u{03C6}', '\u{03C7}', '\u{03C8}', '\u{03C9}',
];
const UP_GREEK: &[char] = &[
  '\u{0391}', '\u{0392}', '\u{0393}', '\u{0394}', '\u{0395}', '\u{0396}', '\u{0397}', '\u{0398}',
  '\u{0399}', '\u{039A}', '\u{039B}', '\u{039C}', '\u{039D}', '\u{039E}', '\u{039F}', '\u{03A0}',
  '\u{03A1}', '\u{03A3}', '\u{03A4}', '\u{03A5}', '\u{03A6}', '\u{03A7}', '\u{03A8}', '\u{03A9}',
];

/// (Internal) Converts the number into one of the char symbols
pub fn radix_format(mut number: i64, symbols: &[char]) -> String {
  let mut chars: Vec<char> = Vec::new();
  let max = symbols.len() as i64;
  while number > 0 {
    let index = (number - 1) % max;
    chars.push(symbols[index as usize]);
    number = (number - 1) / max;
  }
  chars.into_iter().rev().collect()
}
/// (Internal) Converts the number into one of the str symbols
pub fn radix_format_str(mut number: i64, symbols: &[&str]) -> String {
  let mut parts: Vec<&str> = Vec::new();
  let max = symbols.len() as i64;
  while number > 0 {
    let index = (number - 1) % max;
    parts.push(symbols[index as usize]);
    number = (number - 1) / max;
  }
  parts.into_iter().rev().collect()
}

/// converts the number into one or more lowercase latin letters
pub fn radix_alpha(n: i64) -> String { radix_format(n, LETTERS) }
/// converts the number into one or more uppercase latin letters
pub fn radix_up_alpha(n: i64) -> String { radix_format(n, UP_LETTERS) }
/// converts the number into one or more lowercase greek letters
pub fn radix_greek(n: i64) -> String { radix_format(n, GREEK) }

/// converts the number into one or more uppercase greek letters
pub fn radix_up_greek(n: i64) -> String { radix_format(n, UP_GREEK) }

// Dumb place for this, but where else...
// Note: This is one 'The TeX Way'! (bah!! hint: try a large number)
// namely, it's very limited.... what happened to my much-improved version?
const RMLETTERS: &[char] = &['i', 'v', 'x', 'l', 'c', 'd', 'm']; // [CONSTANT]
/// converts the number as a lowercase roman numeral
///
/// Perl parity: `roman(n)` returns the empty string for n <= 0. TeX's
/// `\romannumeral` also produces no output for non-positive input.
pub fn radix_roman(mut n: i64) -> String {
  if n <= 0 {
    return String::new();
  }
  let mut s = String::new();
  let mut div = 1000;
  if n >= div {
    s = (0..(n / div)).map(|_| 'm').collect::<String>();
  }

  let mut p = 4;
  loop {
    n %= div;
    if n == 0 {
      break;
    }
    div /= 10;
    let mut d: i64 = n / div;
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

/// converts the number as a uppercase roman numeral
pub fn radix_up_roman(n: i64) -> String { radix_roman(n).to_uppercase() }

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn roman_non_positive_empty() {
    assert_eq!(radix_roman(0), "");
    assert_eq!(radix_roman(-1), "");
    assert_eq!(radix_roman(i64::MIN), "");
  }

  #[test]
  fn roman_basic_cases() {
    assert_eq!(radix_roman(1), "i");
    assert_eq!(radix_roman(4), "iv");
    assert_eq!(radix_roman(9), "ix");
    assert_eq!(radix_roman(1000), "m");
    assert_eq!(radix_roman(1999), "mcmxcix");
  }

  #[test]
  fn alpha_edge_cases() {
    assert_eq!(radix_alpha(0), "");
    assert_eq!(radix_alpha(-5), "");
    assert_eq!(radix_alpha(1), "a");
    assert_eq!(radix_alpha(26), "z");
    assert_eq!(radix_alpha(27), "aa");
  }

  #[test]
  fn alpha_alphabet_progression() {
    assert_eq!(radix_alpha(28), "ab");
    assert_eq!(radix_alpha(52), "az");
    assert_eq!(radix_alpha(53), "ba");
    // 26*27 = 702 should be the last two-letter (zz).
    assert_eq!(radix_alpha(26 * 26 + 26), "zz");
    assert_eq!(radix_alpha(26 * 26 + 26 + 1), "aaa");
  }

  #[test]
  fn up_alpha_basic() {
    assert_eq!(radix_up_alpha(0), "");
    assert_eq!(radix_up_alpha(1), "A");
    assert_eq!(radix_up_alpha(26), "Z");
    assert_eq!(radix_up_alpha(27), "AA");
  }

  #[test]
  fn up_alpha_vs_alpha_case_only() {
    // For all n, up_alpha(n) should equal alpha(n).to_uppercase().
    for n in 0..60 {
      assert_eq!(
        radix_up_alpha(n),
        radix_alpha(n).to_uppercase(),
        "divergence at {n}"
      );
    }
  }

  #[test]
  fn greek_basic() {
    assert_eq!(radix_greek(0), "");
    assert_eq!(radix_greek(1), "α");
    assert_eq!(radix_greek(24), "ω"); // ω is the 24th (skip final-sigma)
    assert_eq!(radix_greek(25), "αα");
  }

  #[test]
  fn up_greek_basic() {
    assert_eq!(radix_up_greek(0), "");
    assert_eq!(radix_up_greek(1), "Α");
    assert_eq!(radix_up_greek(24), "Ω");
  }

  #[test]
  fn up_roman_cases() {
    assert_eq!(radix_up_roman(0), "");
    assert_eq!(radix_up_roman(1), "I");
    assert_eq!(radix_up_roman(4), "IV");
    assert_eq!(radix_up_roman(1000), "M");
    assert_eq!(radix_up_roman(1999), "MCMXCIX");
  }

  #[test]
  fn radix_format_str_multi_char_symbols() {
    // radix_format_str takes &[&str], useful for abbreviations.
    let syms = &["one", "two", "three"];
    assert_eq!(radix_format_str(0, syms), "");
    assert_eq!(radix_format_str(1, syms), "one");
    assert_eq!(radix_format_str(3, syms), "three");
    // n=4 overflows into second digit: (4-1)%3=0→"one", (4-1)/3=1; (1-1)%3=0→"one" → "oneone"
    assert_eq!(radix_format_str(4, syms), "oneone");
  }

  #[test]
  fn radix_format_custom_symbols() {
    // Single-char radix_format: same generation as alpha but with a
    // user-chosen alphabet.
    let syms = &['A', 'B'];
    assert_eq!(radix_format(1, syms), "A");
    assert_eq!(radix_format(2, syms), "B");
    // n=3 → (3-1)%2=0→'A', (3-1)/2=1→'A' → "AA"
    assert_eq!(radix_format(3, syms), "AA");
    assert_eq!(radix_format(4, syms), "AB");
  }
}
