//! Radix conversion utilities for generating labels and ID suffixes.
//!
//! Port of `LaTeXML::Util::Radix`.
//! Generates labels in the sequence: a,b,...,z,aa,ab,...,az,ba,...,zz,aaa,...

const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";

const GREEK_LOWER: &[char] = &[
  '\u{03B1}', '\u{03B2}', '\u{03B3}', '\u{03B4}', '\u{03B5}', '\u{03B6}', '\u{03B7}', '\u{03B8}',
  '\u{03B9}', '\u{03BA}', '\u{03BB}', '\u{03BC}', '\u{03BD}', '\u{03BE}', '\u{03BF}', '\u{03C0}',
  '\u{03C1}', '\u{03C3}', '\u{03C4}', '\u{03C5}', '\u{03C6}', '\u{03C7}', '\u{03C8}', '\u{03C9}',
];

const GREEK_UPPER: &[char] = &[
  '\u{0391}', '\u{0392}', '\u{0393}', '\u{0394}', '\u{0395}', '\u{0396}', '\u{0397}', '\u{0398}',
  '\u{0399}', '\u{039A}', '\u{039B}', '\u{039C}', '\u{039D}', '\u{039E}', '\u{039F}', '\u{03A0}',
  '\u{03A1}', '\u{03A3}', '\u{03A4}', '\u{03A5}', '\u{03A6}', '\u{03A7}', '\u{03A8}', '\u{03A9}',
];

/// Generic radix formatting: convert a 1-based number into a string
/// using the given symbol set.
///
/// Produces: symbols[0], symbols[1], ..., symbols[n-1],
///           symbols[0]symbols[0], symbols[0]symbols[1], ...
fn radix_format_chars(mut number: u32, symbols: &[char]) -> String {
  let mut result = String::new();
  let base = symbols.len() as u32;
  while number > 0 {
    let idx = ((number - 1) % base) as usize;
    result.insert(0, symbols[idx]);
    number = (number - 1) / base;
  }
  result
}

fn radix_format_bytes(mut number: u32, symbols: &[u8]) -> String {
  let mut result = Vec::new();
  let base = symbols.len() as u32;
  while number > 0 {
    let idx = ((number - 1) % base) as usize;
    result.insert(0, symbols[idx]);
    number = (number - 1) / base;
  }
  String::from_utf8(result).unwrap()
}

/// Convert number to lowercase latin letters: 1→a, 2→b, ..., 26→z, 27→aa, ...
pub fn radix_alpha(n: u32) -> String { radix_format_bytes(n, LOWER) }

/// Convert number to uppercase latin letters: 1→A, 2→B, ..., 26→Z, 27→AA, ...
pub fn radix_alpha_upper(n: u32) -> String { radix_format_bytes(n, UPPER) }

/// Convert number to lowercase greek letters.
pub fn radix_greek(n: u32) -> String { radix_format_chars(n, GREEK_LOWER) }

/// Convert number to uppercase greek letters.
pub fn radix_greek_upper(n: u32) -> String { radix_format_chars(n, GREEK_UPPER) }

/// Convert number to lowercase roman numerals.
pub fn radix_roman(mut n: u32) -> String {
  let letters = ['i', 'v', 'x', 'l', 'c', 'd', 'm'];
  let mut s = String::new();
  let mut div: u32 = 1000;

  // `n >= div` (not `n > div`) — `radix_roman(1000)` must produce "m".
  if n >= div {
    for _ in 0..(n / div) {
      s.push('m');
    }
  }
  let mut p: i32 = 4;

  loop {
    n %= div;
    if n == 0 {
      break;
    }
    div /= 10;
    if div == 0 {
      break;
    }
    let mut d = n / div;
    if d % 5 == 4 {
      s.push(letters[p as usize]);
      d += 1;
    }
    if d > 4 {
      s.push(letters[(p + (d / 5) as i32) as usize]);
      d %= 5;
    }
    for _ in 0..d {
      s.push(letters[p as usize]);
    }
    p -= 2;
    if p < 0 {
      break;
    }
  }
  s
}

/// Convert number to uppercase roman numerals.
pub fn radix_roman_upper(n: u32) -> String { radix_roman(n).to_uppercase() }

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_radix_alpha() {
    assert_eq!(radix_alpha(1), "a");
    assert_eq!(radix_alpha(2), "b");
    assert_eq!(radix_alpha(26), "z");
    assert_eq!(radix_alpha(27), "aa");
    assert_eq!(radix_alpha(28), "ab");
    assert_eq!(radix_alpha(52), "az");
    assert_eq!(radix_alpha(53), "ba");
    assert_eq!(radix_alpha(702), "zz");
    assert_eq!(radix_alpha(703), "aaa");
  }

  #[test]
  fn test_radix_roman() {
    assert_eq!(radix_roman(1), "i");
    assert_eq!(radix_roman(4), "iv");
    assert_eq!(radix_roman(9), "ix");
    assert_eq!(radix_roman(14), "xiv");
    assert_eq!(radix_roman(42), "xlii");
    assert_eq!(radix_roman(1000), "m"); // boundary that was broken (`n > div` bug)
    assert_eq!(radix_roman(1999), "mcmxcix");
    assert_eq!(radix_roman(2000), "mm");
    assert_eq!(radix_roman(3999), "mmmcmxcix");
  }

  #[test]
  fn test_radix_alpha_upper() {
    assert_eq!(radix_alpha_upper(1), "A");
    assert_eq!(radix_alpha_upper(26), "Z");
    assert_eq!(radix_alpha_upper(27), "AA");
    assert_eq!(radix_alpha_upper(703), "AAA");
  }

  #[test]
  fn test_radix_alpha_upper_vs_lower_case() {
    // For all n > 0, upper version is lower version uppercased.
    for n in 1..100 {
      assert_eq!(
        radix_alpha_upper(n),
        radix_alpha(n).to_uppercase(),
        "divergence at n={n}"
      );
    }
  }

  #[test]
  fn test_radix_greek_basic() {
    assert_eq!(radix_greek(1), "α");
    assert_eq!(radix_greek(2), "β");
    // 24th symbol is ω (the medial sigma was skipped).
    assert_eq!(radix_greek(24), "ω");
    // n=25 wraps: αα.
    assert_eq!(radix_greek(25), "αα");
  }

  #[test]
  fn test_radix_greek_upper_basic() {
    assert_eq!(radix_greek_upper(1), "Α");
    assert_eq!(radix_greek_upper(24), "Ω");
    assert_eq!(radix_greek_upper(25), "ΑΑ");
  }

  #[test]
  fn test_radix_roman_upper() {
    assert_eq!(radix_roman_upper(1), "I");
    assert_eq!(radix_roman_upper(4), "IV");
    assert_eq!(radix_roman_upper(1999), "MCMXCIX");
  }

  #[test]
  fn test_radix_alpha_zero_is_empty() {
    // Zero produces empty; values > 0 always produce non-empty.
    assert_eq!(radix_alpha(0), "");
    assert_eq!(radix_alpha_upper(0), "");
    assert_eq!(radix_greek(0), "");
    assert_eq!(radix_roman(0), "");
  }
}
