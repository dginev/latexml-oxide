//! MathML Operator Dictionary.
//!
//! Port of `LaTeXML::Post::MathML::OperatorDictionary` (252 lines of Perl).
//! Implements the MathML Core operator dictionary algorithm (Section 2.6.1).
//! Maps (operator-content, form) → category → spacing and properties.

/// Spacing constants (in em units, matching MathML Core).
pub const THIN: f64 = 0.167; // 3 mu
pub const MED: f64 = 0.222; // 4 mu
pub const THICK: f64 = 0.278; // 5 mu

/// Properties returned by the operator dictionary lookup.
#[derive(Debug, Clone, Default)]
pub struct OpDictProperties {
  pub lspace:        f64,
  pub rspace:        f64,
  pub stretchy:      bool,
  pub symmetric:     bool,
  pub largeop:       bool,
  pub movablelimits: bool,
  pub fence:         bool,
  pub separator:     bool,
}

/// Map LaTeXML role to MathML form.
///
/// Port of `$role_form`.
pub fn role_to_form(role: &str) -> &'static str {
  match role {
    "OPEN" => "prefix",
    "CLOSE" | "VERTBAR" | "PERIOD" | "POSTFIX" => "postfix",
    "OPFUNCTION" | "TRIGFUNCTION" | "BIGOP" | "SUMOP" | "INTOP" | "LIMITOP" | "OPERATOR"
    | "DIFFOP" => "prefix",
    _ => "infix",
  }
}

/// Look up operator properties from the MathML Core dictionary.
///
/// Port of `opdict_lookup`.
/// Returns properties including spacing, stretchy, fence, separator, etc.
pub fn opdict_lookup(content: &str, role: &str) -> OpDictProperties {
  let form = role_to_form(role);
  let category = lookup_category(content, form);

  let mut props = category_to_properties(category);

  // Add fence/separator from special tables
  if is_fence_operator(content) {
    props.fence = true;
  }
  if is_separator_operator(content) {
    props.separator = true;
  }

  props
}

/// Determine the operator category from content and form.
///
/// Port of `lookup_category` — implements MathML Core Section 2.6.1 algorithm.
fn lookup_category(content: &str, form: &str) -> &'static str {
  let chars: Vec<char> = content.chars().collect();
  let len = chars.len();

  // (1) Length > 2: Default
  if len > 2 {
    return "Default";
  }
  if len == 0 {
    return "Default";
  }

  let code1 = chars[0] as u32;

  // (2) Single character in U+0320–U+03FF: Default
  if len == 1 && (0x0320..=0x03FF).contains(&code1) {
    return "Default";
  }

  // Two-character handling
  let effective_code = if len == 2 {
    let code2 = chars[1] as u32;
    // Combining overlays: strip to first character
    if code2 == 0x0338 || code2 == 0x20D2 {
      code1
    }
    // Two-ASCII-char operators: map to pseudo-codepoint
    else if let Some(idx) = two_ascii_char_index(content) {
      0x0320 + idx as u32
    } else {
      return "Default";
    }
  } else {
    code1
  };

  // (3) Special case: | and ~ in infix form
  if form == "infix" && (effective_code == 0x007C || effective_code == 0x223C) {
    return "ForceDefault";
  }

  // Look up in the content-form tables
  lookup_content_form(effective_code, form).unwrap_or("Default")
}

/// Two-ASCII-character operators (Perl's Operators_2_ascii_chars).
fn two_ascii_char_index(content: &str) -> Option<usize> {
  const OPS: &[&str] = &[
    "!!", "!=", "&&", "**", "*=", "++", "+=", "--", "-=", "->", "//", "/=", ":=", "<=", "<>", "==",
    ">=", "||",
  ];
  OPS.iter().position(|&op| op == content)
}

/// Look up category for a codepoint + form in the content-form table.
///
/// Port of `$Content_form` tables.
fn lookup_content_form(code: u32, form: &str) -> Option<&'static str> {
  match form {
    "infix" => lookup_infix(code),
    "prefix" => lookup_prefix(code),
    "postfix" => lookup_postfix(code),
    _ => None,
  }
}

fn lookup_infix(code: u32) -> Option<&'static str> {
  // Category A: arrows (infix)
  if (0x2190..=0x2195).contains(&code)
    || (0x219A..=0x21AE).contains(&code)
    || (0x21B0..=0x21B5).contains(&code)
    || (0x21BC..=0x21D5).contains(&code)
    || (0x21DA..=0x21FF).contains(&code)
    || (0x27F0..=0x27FF).contains(&code)
    || (0x2900..=0x2920).contains(&code)
    || (0x2934..=0x2937).contains(&code)
    || (0x2942..=0x2975).contains(&code)
    || (0x297C..=0x297F).contains(&code)
    || (0x2B04..=0x2B07).contains(&code)
    || (0x2B30..=0x2B4C).contains(&code)
    || (0x2B60..=0x2B65).contains(&code)
    || (0x2B80..=0x2B87).contains(&code)
    || code == 0x2B95
  {
    return Some("A");
  }
  // Category B: additive operators
  if code == 0x002B
    || code == 0x002D
    || code == 0x002F
    || code == 0x00B1
    || code == 0x00F7
    || code == 0x2044
    || (0x2212..=0x2216).contains(&code)
    || (0x2227..=0x222A).contains(&code)
    || code == 0x2236
    || code == 0x2238
    || (0x228C..=0x228E).contains(&code)
    || (0x2293..=0x2296).contains(&code)
    || code == 0x2298
    || (0x229D..=0x229F).contains(&code)
    || (0x22BB..=0x22BD).contains(&code)
    || (0x22CE..=0x22CF).contains(&code)
    || (0x22D2..=0x22D3).contains(&code)
    || (0x2795..=0x2797).contains(&code)
    || (0x2A1F..=0x2A2E).contains(&code)
    || (0x2A38..=0x2A3A).contains(&code)
    || code == 0x2A3E
    || (0x2A40..=0x2A63).contains(&code)
    || code == 0x2ADB
    || code == 0x2AF6
    || code == 0x2AFB
    || code == 0x2AFD
  {
    return Some("B");
  }
  // Category C: multiplicative operators
  if code == 0x0025
    || code == 0x002A
    || code == 0x002E
    || (0x003F..=0x0040).contains(&code)
    || code == 0x005E
    || code == 0x00B7
    || code == 0x00D7
    || code == 0x2022
    || code == 0x2043
    || (0x2217..=0x2219).contains(&code)
    || code == 0x2240
    || code == 0x2297
    || (0x2299..=0x229B).contains(&code)
    || (0x22A0..=0x22A1).contains(&code)
    || code == 0x22BA
    || (0x22C4..=0x22CC).contains(&code)
    || (0x2305..=0x2306).contains(&code)
    || code == 0x27CB
    || code == 0x27CD
    || (0x2A1D..=0x2A1E).contains(&code)
    || (0x2A2F..=0x2A3D).contains(&code)
    || code == 0x2A3F
    || code == 0x2A50
    || (0x2A64..=0x2A65).contains(&code)
  {
    return Some("C");
  }
  // Category K: invisible operators
  if code == 0x005C || code == 0x005F || (0x2061..=0x2064).contains(&code) || code == 0x2206 {
    return Some("K");
  }
  // Category M: comma, colon, semicolon
  if code == 0x002C || code == 0x003A || code == 0x003B {
    return Some("M");
  }
  None
}

fn lookup_prefix(code: u32) -> Option<&'static str> {
  // Category D: prefix operators (¬, ∀, ∃, ∇, ±, etc.)
  if code == 0x0021
    || code == 0x002B
    || code == 0x002D
    || code == 0x00AC
    || code == 0x00B1
    || (0x2200..=0x2201).contains(&code)
    || (0x2203..=0x2204).contains(&code)
    || code == 0x2207
    || (0x2212..=0x2213).contains(&code)
    || (0x221F..=0x2222).contains(&code)
    || (0x2234..=0x2235).contains(&code)
    || code == 0x223C
    || (0x22BE..=0x22BF).contains(&code)
    || code == 0x2310
    || code == 0x2319
  {
    return Some("D");
  }
  // Category F: open fences (prefix stretchy symmetric)
  if code == 0x0028
    || code == 0x005B
    || code == 0x007B
    || code == 0x007C
    || code == 0x2016
    || code == 0x2308
    || code == 0x230A
    || code == 0x2329
    || code == 0x2772
    || (0x27E6..=0x27EF).contains(&code)
    || code == 0x2980
    || (0x2983..=0x2999).contains(&code)
    || code == 0x29D8
    || code == 0x29DA
    || code == 0x29FC
  {
    return Some("F");
  }
  // Category H: integrals (prefix stretchy symmetric largeop)
  if (0x222B..=0x2233).contains(&code) || (0x2A0B..=0x2A1C).contains(&code) {
    return Some("H");
  }
  // Category J: sums/products (prefix symmetric largeop movablelimits)
  if (0x220F..=0x2211).contains(&code)
    || (0x22C0..=0x22C3).contains(&code)
    || (0x2A00..=0x2A0A).contains(&code)
    || (0x2A1D..=0x2A1E).contains(&code)
    || code == 0x2AFC
    || code == 0x2AFF
  {
    return Some("J");
  }
  // Category L: differentials, roots
  if (0x2145..=0x2146).contains(&code) || code == 0x2202 || (0x221A..=0x221C).contains(&code) {
    return Some("L");
  }
  None
}

fn lookup_postfix(code: u32) -> Option<&'static str> {
  // Category E: postfix (!, %, degree, primes, etc.)
  if (0x0021..=0x0022).contains(&code)
    || (0x0025..=0x0027).contains(&code)
    || code == 0x0060
    || code == 0x00A8
    || code == 0x00B0
    || (0x00B2..=0x00B4).contains(&code)
    || (0x00B8..=0x00B9).contains(&code)
    || (0x02CA..=0x02CB).contains(&code)
    || (0x02D8..=0x02DA).contains(&code)
    || code == 0x02DD
    || code == 0x0311
    || code == 0x0325
    || code == 0x0327
    || (0x2032..=0x2037).contains(&code)
    || code == 0x2057
    || (0x20DB..=0x20DC).contains(&code)
    || code == 0x23CD
  {
    return Some("E");
  }
  // Category G: close fences (postfix stretchy symmetric)
  if code == 0x0029
    || code == 0x005D
    || code == 0x007C
    || code == 0x007D
    || code == 0x2016
    || code == 0x2309
    || code == 0x230B
    || code == 0x232A
    || code == 0x2773
    || code == 0x27E7
    || code == 0x27E9
    || code == 0x27EB
    || code == 0x27ED
    || code == 0x27EF
    || code == 0x2980
    || (0x2984..=0x2998).contains(&code)
    || code == 0x2999
    || code == 0x29D9
    || code == 0x29DB
    || code == 0x29FD
  {
    return Some("G");
  }
  // Category I: postfix stretchy accents
  if (0x005E..=0x005F).contains(&code)
    || code == 0x007E
    || code == 0x00AF
    || (0x02C6..=0x02C7).contains(&code)
    || code == 0x02C9
    || code == 0x02CD
    || code == 0x02DC
    || code == 0x02F7
    || code == 0x0302
    || code == 0x203E
    || (0x2322..=0x2323).contains(&code)
    || (0x23B4..=0x23B5).contains(&code)
    || (0x23DC..=0x23E1).contains(&code)
  {
    return Some("I");
  }
  None
}

/// Map category name to operator properties.
///
/// Port of `$Category_data`.
fn category_to_properties(category: &str) -> OpDictProperties {
  match category {
    "A" => OpDictProperties {
      lspace: THICK,
      rspace: THICK,
      stretchy: true,
      ..Default::default()
    },
    "B" => OpDictProperties {
      lspace: MED,
      rspace: MED,
      ..Default::default()
    },
    "C" => OpDictProperties {
      lspace: THIN,
      rspace: THIN,
      ..Default::default()
    },
    "D" => OpDictProperties {
      lspace: 0.0,
      rspace: 0.0,
      ..Default::default()
    },
    "E" => OpDictProperties {
      lspace: 0.0,
      rspace: 0.0,
      ..Default::default()
    },
    "F" => OpDictProperties {
      lspace: 0.0,
      rspace: 0.0,
      stretchy: true,
      symmetric: true,
      ..Default::default()
    },
    "G" => OpDictProperties {
      lspace: 0.0,
      rspace: 0.0,
      stretchy: true,
      symmetric: true,
      ..Default::default()
    },
    "H" => OpDictProperties {
      lspace: THIN,
      rspace: THIN,
      symmetric: true,
      largeop: true,
      ..Default::default()
    },
    "I" => OpDictProperties {
      lspace: 0.0,
      rspace: 0.0,
      stretchy: true,
      ..Default::default()
    },
    "J" => OpDictProperties {
      lspace: THIN,
      rspace: THIN,
      symmetric: true,
      largeop: true,
      movablelimits: true,
      ..Default::default()
    },
    "K" => OpDictProperties {
      lspace: 0.0,
      rspace: 0.0,
      ..Default::default()
    },
    "L" => OpDictProperties {
      lspace: THIN,
      rspace: 0.0,
      ..Default::default()
    },
    "M" => OpDictProperties {
      lspace: 0.0,
      rspace: THIN,
      ..Default::default()
    },
    _ => OpDictProperties {
      lspace: THICK,
      rspace: THICK,
      ..Default::default()
    }, // Default & ForceDefault
  }
}

/// Check if a character is in the MathML Core fence operators table.
fn is_fence_operator(content: &str) -> bool {
  let c = match content.chars().next() {
    Some(c) => c as u32,
    None => return false,
  };
  (0x0028..=0x0029).contains(&c)
    || c == 0x005B
    || c == 0x005D
    || (0x007B..=0x007D).contains(&c)
    || c == 0x2016
    || (0x2018..=0x2019).contains(&c)
    || (0x201C..=0x201D).contains(&c)
    || (0x2308..=0x230B).contains(&c)
    || (0x2329..=0x232A).contains(&c)
    || (0x2772..=0x2773).contains(&c)
    || (0x27E6..=0x27EF).contains(&c)
    || c == 0x2980
    || (0x2983..=0x2999).contains(&c)
    || (0x29D8..=0x29DB).contains(&c)
    || (0x29FC..=0x29FD).contains(&c)
}

/// Check if a character is a separator (comma, semicolon, invisible separator).
fn is_separator_operator(content: &str) -> bool {
  let c = match content.chars().next() {
    Some(c) => c as u32,
    None => return false,
  };
  c == 0x002C || c == 0x003B || c == 0x2063
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_opdict_lookup_plus() {
    let props = opdict_lookup("+", "ADDOP");
    assert!((props.lspace - MED).abs() < 0.001); // Category B
    assert!((props.rspace - MED).abs() < 0.001);
    assert!(!props.stretchy);
  }

  #[test]
  fn test_opdict_lookup_open_paren() {
    let props = opdict_lookup("(", "OPEN");
    assert!((props.lspace - 0.0).abs() < 0.001); // Category F
    assert!(props.stretchy);
    assert!(props.symmetric);
    assert!(props.fence);
  }

  #[test]
  fn test_opdict_lookup_integral() {
    let props = opdict_lookup("\u{222B}", "INTOP");
    assert!(props.largeop); // Category H
    assert!(props.symmetric);
  }

  #[test]
  fn test_opdict_lookup_sum() {
    let props = opdict_lookup("\u{2211}", "SUMOP");
    assert!(props.largeop); // Category J
    assert!(props.movablelimits);
  }

  #[test]
  fn test_opdict_lookup_comma() {
    let props = opdict_lookup(",", "PUNCT");
    assert!(props.separator);
    assert!((props.rspace - THIN).abs() < 0.001); // Category M
  }

  #[test]
  fn test_opdict_lookup_invisible_times() {
    let props = opdict_lookup("\u{2062}", "MULOP");
    assert!((props.lspace - 0.0).abs() < 0.001); // Category K
    assert!((props.rspace - 0.0).abs() < 0.001);
  }

  #[test]
  fn test_two_ascii_char_ops() {
    assert_eq!(two_ascii_char_index("!="), Some(1));
    assert_eq!(two_ascii_char_index("->"), Some(9));
    assert_eq!(two_ascii_char_index("xx"), None);
  }
}
