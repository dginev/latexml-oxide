//! Unicode character mapping for mathematical typesetting.
//!
//! Port of `LaTeXML::Util::Unicode` (~350 lines).
//! Maps base characters to Unicode Plane 1 Mathematical Alphanumeric Symbols
//! (U+1D400–U+1D7FF) based on mathvariant style. Also handles superscript/subscript
//! mappings and font name normalization to MathML mathvariant values.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Build a Plane 1 character mapping from base addresses.
///
/// Port of Perl's `makePlane1Map($latin, $GREEK, $greek, $digits)`.
/// Creates mappings for:
/// - Latin uppercase A-Z (26 chars) at `latin + 0..25`
/// - Latin lowercase a-z (26 chars) at `latin + 26..51`
/// - Greek uppercase Α-Ω (25 chars) at `greek_upper + 0..24` (if Some)
/// - Greek lowercase α-ω (25 chars) at `greek_lower + 0..24` (if Some)
/// - Digits 0-9 (10 chars) at `digits + 0..9` (if Some)
fn make_plane1_map(
  latin: u32,
  greek_upper: Option<u32>,
  greek_lower: Option<u32>,
  digits: Option<u32>,
) -> HashMap<char, char> {
  let mut map = HashMap::new();
  // Latin uppercase A-Z
  for i in 0..26u32 {
    let src = char::from_u32('A' as u32 + i).unwrap();
    let dst = char::from_u32(latin + i).unwrap();
    map.insert(src, dst);
  }
  // Latin lowercase a-z
  for i in 0..26u32 {
    let src = char::from_u32('a' as u32 + i).unwrap();
    let dst = char::from_u32(latin + 26 + i).unwrap();
    map.insert(src, dst);
  }
  // Greek uppercase Α(U+0391) through Ω — 25 characters
  if let Some(base) = greek_upper {
    for i in 0..25u32 {
      let src = char::from_u32(0x0391 + i).unwrap();
      let dst = char::from_u32(base + i).unwrap();
      map.insert(src, dst);
    }
  }
  // Greek lowercase α(U+03B1) through ω — 25 characters
  if let Some(base) = greek_lower {
    for i in 0..25u32 {
      let src = char::from_u32(0x03B1 + i).unwrap();
      let dst = char::from_u32(base + i).unwrap();
      map.insert(src, dst);
    }
  }
  // Digits 0-9
  if let Some(base) = digits {
    for i in 0..10u32 {
      let src = char::from_u32('0' as u32 + i).unwrap();
      let dst = char::from_u32(base + i).unwrap();
      map.insert(src, dst);
    }
  }
  map
}

/// All Plane 1 Unicode mapping tables, keyed by mathvariant style name.
///
/// Port of Perl's `%unicode_map` hash.
static UNICODE_MAP: LazyLock<HashMap<&'static str, HashMap<char, char>>> = LazyLock::new(|| {
  let mut map: HashMap<&'static str, HashMap<char, char>> = HashMap::new();

  // bold: full set (Latin + Greek + digits)
  map.insert(
    "bold",
    make_plane1_map(0x1D400, Some(0x1D6A8), Some(0x1D6C2), Some(0x1D7CE)),
  );

  // italic: Latin + Greek, no digits. Special: h => PLANCK CONSTANT
  {
    let mut m = make_plane1_map(0x1D434, Some(0x1D6E2), Some(0x1D6FC), None);
    m.insert('h', '\u{210E}');
    map.insert("italic", m);
  }

  // bold-italic: Latin + Greek, no digits
  map.insert(
    "bold-italic",
    make_plane1_map(0x1D468, Some(0x1D71C), Some(0x1D736), None),
  );

  // sans-serif: Latin + digits, no Greek
  map.insert(
    "sans-serif",
    make_plane1_map(0x1D5A0, None, None, Some(0x1D7E2)),
  );

  // bold-sans-serif: full set
  map.insert(
    "bold-sans-serif",
    make_plane1_map(0x1D5D4, Some(0x1D756), Some(0x1D770), Some(0x1D7EC)),
  );

  // sans-serif-italic: Latin only
  map.insert(
    "sans-serif-italic",
    make_plane1_map(0x1D608, None, None, None),
  );

  // sans-serif-bold-italic: Latin + Greek, no digits
  map.insert(
    "sans-serif-bold-italic",
    make_plane1_map(0x1D63C, Some(0x1D790), Some(0x1D7AA), None),
  );

  // monospace: Latin + digits, no Greek
  map.insert(
    "monospace",
    make_plane1_map(0x1D670, None, None, Some(0x1D7F6)),
  );

  // script: Latin only, with 11 special character overrides
  {
    let mut m = make_plane1_map(0x1D49C, None, None, None);
    m.insert('B', '\u{212C}'); // SCRIPT CAPITAL B
    m.insert('E', '\u{2130}'); // SCRIPT CAPITAL E
    m.insert('F', '\u{2131}'); // SCRIPT CAPITAL F
    m.insert('H', '\u{210B}'); // SCRIPT CAPITAL H
    m.insert('I', '\u{2110}'); // SCRIPT CAPITAL I
    m.insert('L', '\u{2112}'); // SCRIPT CAPITAL L
    m.insert('M', '\u{2133}'); // SCRIPT CAPITAL M
    m.insert('R', '\u{211B}'); // SCRIPT CAPITAL R
    m.insert('e', '\u{212F}'); // SCRIPT SMALL E
    m.insert('g', '\u{210A}'); // SCRIPT SMALL G
    m.insert('o', '\u{2134}'); // SCRIPT SMALL O
    map.insert("script", m);
  }

  // bold-script: Latin only
  map.insert("bold-script", make_plane1_map(0x1D4D0, None, None, None));

  // fraktur: Latin only, with 5 special character overrides
  {
    let mut m = make_plane1_map(0x1D504, None, None, None);
    m.insert('C', '\u{212D}'); // FRAKTUR CAPITAL C
    m.insert('H', '\u{210C}'); // FRAKTUR CAPITAL H
    m.insert('I', '\u{2111}'); // FRAKTUR CAPITAL I
    m.insert('R', '\u{211C}'); // FRAKTUR CAPITAL R
    m.insert('Z', '\u{2128}'); // FRAKTUR CAPITAL Z
    map.insert("fraktur", m);
  }

  // bold-fraktur: Latin only
  map.insert("bold-fraktur", make_plane1_map(0x1D56C, None, None, None));

  // double-struck: Latin + digits, with 7 special character overrides
  {
    let mut m = make_plane1_map(0x1D538, None, None, Some(0x1D7D8));
    m.insert('C', '\u{2102}'); // DOUBLE-STRUCK CAPITAL C (complex numbers)
    m.insert('H', '\u{210D}'); // DOUBLE-STRUCK CAPITAL H (quaternions)
    m.insert('N', '\u{2115}'); // DOUBLE-STRUCK CAPITAL N (natural numbers)
    m.insert('P', '\u{2119}'); // DOUBLE-STRUCK CAPITAL P
    m.insert('Q', '\u{211A}'); // DOUBLE-STRUCK CAPITAL Q (rationals)
    m.insert('R', '\u{211D}'); // DOUBLE-STRUCK CAPITAL R (reals)
    m.insert('Z', '\u{2124}'); // DOUBLE-STRUCK CAPITAL Z (integers)
    map.insert("double-struck", m);
  }

  // superscript: scattered Unicode characters
  {
    let mut m = HashMap::new();
    m.insert('\u{2032}', '\''); // \prime
    m.insert('0', '\u{2070}');
    m.insert('1', '\u{00B9}');
    m.insert('2', '\u{00B2}');
    m.insert('3', '\u{00B3}');
    m.insert('4', '\u{2074}');
    m.insert('5', '\u{2075}');
    m.insert('6', '\u{2076}');
    m.insert('7', '\u{2077}');
    m.insert('8', '\u{2078}');
    m.insert('9', '\u{2079}');
    m.insert('+', '\u{207A}');
    m.insert('-', '\u{207B}');
    m.insert('=', '\u{207C}');
    m.insert('(', '\u{207D}');
    m.insert(')', '\u{207E}');
    m.insert('n', '\u{207F}');
    m.insert('i', '\u{2071}');
    m.insert('V', '\u{2C7D}');
    m.insert('h', '\u{02B0}');
    m.insert('j', '\u{02B2}');
    m.insert('r', '\u{02B3}');
    m.insert('w', '\u{02B7}');
    m.insert('y', '\u{02B8}');
    m.insert('s', '\u{02E2}');
    m.insert('x', '\u{02E3}');
    m.insert('A', '\u{1D2C}');
    m.insert('\u{00C6}', '\u{1D2D}'); // Æ
    m.insert('B', '\u{1D2E}');
    m.insert('D', '\u{1D30}');
    m.insert('E', '\u{1D31}');
    m.insert('G', '\u{1D33}');
    m.insert('H', '\u{1D34}');
    m.insert('I', '\u{1D35}');
    m.insert('J', '\u{1D36}');
    m.insert('K', '\u{1D37}');
    m.insert('L', '\u{1D38}');
    m.insert('M', '\u{1D39}');
    m.insert('N', '\u{1D3A}');
    m.insert('O', '\u{1D3C}');
    m.insert('P', '\u{1D3E}');
    m.insert('R', '\u{1D3F}');
    m.insert('T', '\u{1D40}');
    m.insert('U', '\u{1D41}');
    m.insert('W', '\u{1D42}');
    m.insert('a', '\u{1D43}');
    m.insert('\u{03B1}', '\u{1D45}'); // α
    m.insert('\u{00E6}', '\u{1D46}'); // æ
    m.insert('b', '\u{1D47}');
    m.insert('d', '\u{1D48}');
    m.insert('e', '\u{1D49}');
    m.insert('\u{03B5}', '\u{1D4B}'); // ε (varepsilon)
    m.insert('\u{03F5}', '\u{1D4B}'); // ϵ (epsilon) — close enough
    m.insert('g', '\u{1D4D}');
    m.insert('!', '\u{1D4E}');
    m.insert('k', '\u{1D4F}');
    m.insert('m', '\u{1D50}');
    m.insert('o', '\u{1D52}');
    m.insert('p', '\u{1D56}');
    m.insert('t', '\u{1D57}');
    m.insert('u', '\u{1D58}');
    m.insert('v', '\u{1D5B}');
    m.insert('\u{03B2}', '\u{1D5D}'); // β
    m.insert('\u{03B3}', '\u{1D5E}'); // γ
    m.insert('\u{03B4}', '\u{1D5F}'); // δ
    m.insert('\u{03C6}', '\u{1D60}'); // φ (varphi)
    m.insert('\u{03D5}', '\u{1D60}'); // ϕ (phi) — close enough
    m.insert('\u{03BE}', '\u{1D61}'); // ξ
    m.insert('c', '\u{1D9C}');
    m.insert('f', '\u{1DA0}');
    m.insert('\u{03A6}', '\u{1DB2}'); // Φ
    m.insert('\u{03C5}', '\u{1DB7}'); // υ (upsilon)
    m.insert('z', '\u{1DBB}');
    m.insert('\u{03B8}', '\u{1DBF}'); // θ
    map.insert("superscript", m);
  }

  // subscript: scattered Unicode characters
  {
    let mut m = HashMap::new();
    m.insert('0', '\u{2080}');
    m.insert('1', '\u{2081}');
    m.insert('2', '\u{2082}');
    m.insert('3', '\u{2083}');
    m.insert('4', '\u{2084}');
    m.insert('5', '\u{2085}');
    m.insert('6', '\u{2086}');
    m.insert('7', '\u{2087}');
    m.insert('8', '\u{2088}');
    m.insert('9', '\u{2089}');
    m.insert('+', '\u{208A}');
    m.insert('-', '\u{208B}');
    m.insert('=', '\u{208C}');
    m.insert('(', '\u{208D}');
    m.insert(')', '\u{208E}');
    m.insert('a', '\u{2090}');
    m.insert('e', '\u{2091}');
    m.insert('o', '\u{2092}');
    m.insert('x', '\u{2093}');
    // 'upsidedowne' => "\x{2094}" — not a single char, skipped
    m.insert('h', '\u{2095}');
    m.insert('k', '\u{2096}');
    m.insert('l', '\u{2097}');
    m.insert('m', '\u{2098}');
    m.insert('n', '\u{2099}');
    m.insert('p', '\u{209A}');
    m.insert('s', '\u{209B}');
    m.insert('t', '\u{209C}');
    m.insert('j', '\u{2C7C}');
    m.insert('i', '\u{1D62}');
    m.insert('r', '\u{1D63}');
    m.insert('u', '\u{1D64}');
    m.insert('v', '\u{1D65}');
    m.insert('\u{03B2}', '\u{1D66}'); // β
    m.insert('\u{03B3}', '\u{1D67}'); // γ
    m.insert('\u{03C1}', '\u{1D68}'); // ρ
    m.insert('\u{03C6}', '\u{1D69}'); // φ (varphi)
    m.insert('\u{03D5}', '\u{1D69}'); // ϕ (phi) — close enough
    m.insert('\u{03BE}', '\u{1D6A}'); // ξ
    map.insert("subscript", m);
  }

  map
});

/// Convert a string's characters to the styled Unicode equivalent for the given mathvariant.
///
/// Port of Perl's `unicode_convert($string, $style)`.
/// Returns `None` if any character in the string has no mapping (all-or-nothing semantics).
pub fn unicode_convert(string: &str, style: &str) -> Option<String> {
  let mapping = UNICODE_MAP.get(style)?;
  let mut result = String::with_capacity(string.len() * 4);
  for ch in string.chars() {
    match mapping.get(&ch) {
      Some(&mapped) => result.push(mapped),
      None => return None, // All-or-nothing: if ANY char has no mapping, fail
    }
  }
  Some(result)
}

/// Mathvariant normalization table.
///
/// Port of Perl's `%mathvariants` hash (28 entries → 12 canonical variants).
static MATHVARIANTS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
  let mut m = HashMap::new();
  m.insert("upright", "normal");
  m.insert("serif", "normal");
  m.insert("medium", "normal");
  m.insert("bold", "bold");
  m.insert("italic", "italic");
  m.insert("medium italic", "italic");
  m.insert("bold italic", "bold-italic");
  m.insert("doublestruck", "double-struck");
  m.insert("blackboard", "double-struck");
  m.insert("blackboard bold", "double-struck");
  m.insert("blackboard upright", "double-struck");
  m.insert("blackboard bold upright", "double-struck");
  m.insert("fraktur", "fraktur");
  m.insert("fraktur italic", "fraktur");
  m.insert("fraktur bold", "bold-fraktur");
  m.insert("script", "script");
  m.insert("script italic", "script");
  m.insert("script bold", "bold-script");
  m.insert("caligraphic", "script");
  m.insert("caligraphic bold", "bold-script");
  m.insert("sansserif", "sans-serif");
  m.insert("sansserif bold", "bold-sans-serif");
  m.insert("sansserif italic", "sans-serif-italic");
  m.insert("sansserif bold italic", "sans-serif-bold-italic");
  m.insert("typewriter", "monospace");
  m.insert("typewriter bold", "monospace");
  m.insert("typewriter italic", "monospace");
  m.insert("typewriter bold italic", "monospace");
  m
});

/// Normalize a LaTeXML font name to a MathML mathvariant value.
///
/// Port of Perl's `unicode_mathvariant($font)`.
/// Applies normalization: slanted→italic, removes redundant "serif"/"upright"/"medium",
/// then looks up in the canonical table. Returns "normal" as fallback.
pub fn unicode_mathvariant(font: &str) -> &'static str {
  let mut f = font.to_string();

  // slanted → italic (equivalent in math)
  f = f.replace("slanted", "italic");

  // Remove "serif" unless font is exactly "serif"
  if f != "serif" {
    // Perl: s/(?<!\w)serif// — remove "serif" not preceded by a word char
    // In practice, this removes standalone "serif" or "serif" at start
    f = remove_non_prefixed(&f, "serif");
  }

  // Remove "upright" unless it's the first component
  // Perl: s/(?<!^)upright// — remove "upright" not at start
  if !f.starts_with("upright") {
    f = f.replace("upright", "");
  } else if f.len() > "upright".len() {
    // starts with upright but has more — remove subsequent occurrences
    let (prefix, rest) = f.split_at("upright".len());
    f = format!("{}{}", prefix, rest.replace("upright", ""));
  }

  // Remove "medium" unless it's the first component
  // Perl: s/(?<!^)medium// — remove "medium" not at start
  if !f.starts_with("medium") {
    f = f.replace("medium", "");
  } else if f.len() > "medium".len() {
    let (prefix, rest) = f.split_at("medium".len());
    f = format!("{}{}", prefix, rest.replace("medium", ""));
  }

  // Trim whitespace
  let f = f.trim();

  if let Some(&variant) = MATHVARIANTS.get(f) {
    return variant;
  }
  "normal"
}

/// Remove a word that is not preceded by a word character.
/// Approximation of Perl's `s/(?<!\w)word//`.
fn remove_non_prefixed(s: &str, word: &str) -> String {
  let mut result = String::with_capacity(s.len());
  let mut i = 0;
  let bytes = s.as_bytes();
  let word_bytes = word.as_bytes();
  while i < bytes.len() {
    if i + word_bytes.len() <= bytes.len() && &bytes[i..i + word_bytes.len()] == word_bytes {
      // Check if preceded by a word character
      let preceded_by_word = if i > 0 {
        let prev = bytes[i - 1];
        prev.is_ascii_alphanumeric() || prev == b'_'
      } else {
        false
      };
      if !preceded_by_word {
        i += word_bytes.len();
        continue;
      }
    }
    result.push(bytes[i] as char);
    i += 1;
  }
  result
}

/// Plane1-hackable variants: maps bold variants to their non-bold base for partial conversion.
///
/// Port of Perl's `%plane1hackable`.
/// When `hackplane1` is true, only these variants are converted (bold variants map to non-bold).
pub fn plane1_hackable(variant: &str) -> Option<&'static str> {
  match variant {
    "script" => Some("script"),
    "bold-script" => Some("script"),
    "fraktur" => Some("fraktur"),
    "bold-fraktur" => Some("fraktur"),
    "double-struck" => Some("double-struck"),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_unicode_convert_bold() {
    // "ABC" in bold should map to Mathematical Bold Capital A/B/C
    let result = unicode_convert("ABC", "bold");
    assert_eq!(result, Some("\u{1D400}\u{1D401}\u{1D402}".to_string()));
  }

  #[test]
  fn test_unicode_convert_bold_lowercase() {
    let result = unicode_convert("abc", "bold");
    assert_eq!(result, Some("\u{1D41A}\u{1D41B}\u{1D41C}".to_string()));
  }

  #[test]
  fn test_unicode_convert_italic_h() {
    // italic h → PLANCK CONSTANT (U+210E)
    let result = unicode_convert("h", "italic");
    assert_eq!(result, Some("\u{210E}".to_string()));
  }

  #[test]
  fn test_unicode_convert_script_special() {
    // Script B → U+212C (not the Plane 1 address)
    let result = unicode_convert("B", "script");
    assert_eq!(result, Some("\u{212C}".to_string()));
  }

  #[test]
  fn test_unicode_convert_double_struck_special() {
    // Double-struck R → U+211D (real numbers)
    let result = unicode_convert("R", "double-struck");
    assert_eq!(result, Some("\u{211D}".to_string()));
  }

  #[test]
  fn test_unicode_convert_fraktur_special() {
    let result = unicode_convert("C", "fraktur");
    assert_eq!(result, Some("\u{212D}".to_string()));
  }

  #[test]
  fn test_unicode_convert_all_or_nothing() {
    // "A1" with italic: A maps but 1 doesn't (italic has no digits)
    let result = unicode_convert("A1", "italic");
    assert_eq!(result, None);
  }

  #[test]
  fn test_unicode_convert_bold_digits() {
    // "1" = digit 1 → 0x1D7CE + 1 = 0x1D7CF, "2" → 0x1D7D0, "3" → 0x1D7D1
    let result = unicode_convert("123", "bold");
    assert_eq!(result, Some("\u{1D7CF}\u{1D7D0}\u{1D7D1}".to_string()));
  }

  #[test]
  fn test_unicode_convert_bold_greek() {
    // Bold Alpha (Α U+0391) → U+1D6A8
    let result = unicode_convert("\u{0391}", "bold");
    assert_eq!(result, Some("\u{1D6A8}".to_string()));
  }

  #[test]
  fn test_unicode_convert_empty_string() {
    let result = unicode_convert("", "bold");
    assert_eq!(result, Some(String::new()));
  }

  #[test]
  fn test_unicode_convert_unknown_style() {
    let result = unicode_convert("A", "nonexistent");
    assert_eq!(result, None);
  }

  #[test]
  fn test_unicode_mathvariant_basic() {
    assert_eq!(unicode_mathvariant("italic"), "italic");
    assert_eq!(unicode_mathvariant("bold"), "bold");
    assert_eq!(unicode_mathvariant("bold italic"), "bold-italic");
    assert_eq!(unicode_mathvariant("typewriter"), "monospace");
    assert_eq!(unicode_mathvariant("caligraphic"), "script");
  }

  #[test]
  fn test_unicode_mathvariant_normalization() {
    // slanted → italic
    assert_eq!(unicode_mathvariant("slanted"), "italic");
    // "medium italic" → "italic"
    assert_eq!(unicode_mathvariant("medium italic"), "italic");
    // "serif" alone → "normal"
    assert_eq!(unicode_mathvariant("serif"), "normal");
    // Unknown → "normal"
    assert_eq!(unicode_mathvariant("unknown_font"), "normal");
  }

  #[test]
  fn test_unicode_mathvariant_compound() {
    assert_eq!(
      unicode_mathvariant("sansserif bold italic"),
      "sans-serif-bold-italic"
    );
    assert_eq!(unicode_mathvariant("blackboard bold"), "double-struck");
    assert_eq!(unicode_mathvariant("fraktur bold"), "bold-fraktur");
  }

  #[test]
  fn test_plane1_hackable() {
    assert_eq!(plane1_hackable("script"), Some("script"));
    assert_eq!(plane1_hackable("bold-script"), Some("script"));
    assert_eq!(plane1_hackable("bold"), None);
    assert_eq!(plane1_hackable("italic"), None);
  }
}
