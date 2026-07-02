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

// === generated from Perl $Content_form / $Operators_fence (verbatim) ===
const INFIX_TABLE: &[(u32, u32, &str)] = &[
  (0x0025, 0x0025, "C"),
  (0x002A, 0x002A, "C"),
  (0x002B, 0x002B, "B"),
  (0x002C, 0x002C, "M"),
  (0x002D, 0x002D, "B"),
  (0x002E, 0x002E, "C"),
  (0x002F, 0x002F, "B"),
  (0x003A, 0x003A, "M"),
  (0x003B, 0x003B, "M"),
  (0x003F, 0x0040, "C"),
  (0x005C, 0x005C, "K"),
  (0x005E, 0x005E, "C"),
  (0x005F, 0x005F, "K"),
  (0x00B1, 0x00B1, "B"),
  (0x00B7, 0x00B7, "C"),
  (0x00D7, 0x00D7, "C"),
  (0x00F7, 0x00F7, "B"),
  (0x0322, 0x0322, "B"),
  (0x0323, 0x0323, "C"),
  (0x032E, 0x032E, "C"),
  (0x2022, 0x2022, "C"),
  (0x2043, 0x2043, "C"),
  (0x2044, 0x2044, "B"),
  (0x2061, 0x2064, "K"),
  (0x2190, 0x2195, "A"),
  (0x219A, 0x21AE, "A"),
  (0x21B0, 0x21B5, "A"),
  (0x21B9, 0x21B9, "A"),
  (0x21BC, 0x21D5, "A"),
  (0x21DA, 0x21F0, "A"),
  (0x21F3, 0x21FF, "A"),
  (0x2206, 0x2206, "K"),
  (0x2212, 0x2216, "B"),
  (0x2217, 0x2219, "C"),
  (0x2227, 0x222A, "B"),
  (0x2236, 0x2236, "B"),
  (0x2238, 0x2238, "B"),
  (0x2240, 0x2240, "C"),
  (0x228C, 0x228E, "B"),
  (0x2293, 0x2296, "B"),
  (0x2297, 0x2297, "C"),
  (0x2298, 0x2298, "B"),
  (0x2299, 0x229B, "C"),
  (0x229D, 0x229F, "B"),
  (0x22A0, 0x22A1, "C"),
  (0x22BA, 0x22BA, "C"),
  (0x22BB, 0x22BD, "B"),
  (0x22C4, 0x22C7, "C"),
  (0x22C9, 0x22CC, "C"),
  (0x22CE, 0x22CF, "B"),
  (0x22D2, 0x22D3, "B"),
  (0x2305, 0x2306, "C"),
  (0x2794, 0x2794, "A"),
  (0x2795, 0x2797, "B"),
  (0x2799, 0x2799, "A"),
  (0x279B, 0x27A1, "A"),
  (0x27A5, 0x27A6, "A"),
  (0x27A8, 0x27AF, "A"),
  (0x27B1, 0x27B1, "A"),
  (0x27B3, 0x27B3, "A"),
  (0x27B5, 0x27B5, "A"),
  (0x27B8, 0x27B8, "A"),
  (0x27BA, 0x27BE, "A"),
  (0x27CB, 0x27CB, "C"),
  (0x27CD, 0x27CD, "C"),
  (0x27F0, 0x27F1, "A"),
  (0x27F4, 0x27FF, "A"),
  (0x2900, 0x2920, "A"),
  (0x2934, 0x2937, "A"),
  (0x2942, 0x2975, "A"),
  (0x297C, 0x297F, "A"),
  (0x29B8, 0x29B8, "B"),
  (0x29BC, 0x29BC, "B"),
  (0x29C4, 0x29C5, "B"),
  (0x29C6, 0x29C8, "C"),
  (0x29D4, 0x29D7, "C"),
  (0x29E2, 0x29E2, "C"),
  (0x29F5, 0x29FB, "B"),
  (0x2A1D, 0x2A1E, "C"),
  (0x2A1F, 0x2A2E, "B"),
  (0x2A2F, 0x2A37, "C"),
  (0x2A38, 0x2A3A, "B"),
  (0x2A3B, 0x2A3D, "C"),
  (0x2A3E, 0x2A3E, "B"),
  (0x2A3F, 0x2A3F, "C"),
  (0x2A40, 0x2A4F, "B"),
  (0x2A50, 0x2A50, "C"),
  (0x2A51, 0x2A63, "B"),
  (0x2A64, 0x2A65, "C"),
  (0x2ADB, 0x2ADB, "B"),
  (0x2ADC, 0x2ADD, "C"),
  (0x2AF6, 0x2AF6, "B"),
  (0x2AFB, 0x2AFB, "B"),
  (0x2AFD, 0x2AFD, "B"),
  (0x2AFE, 0x2AFE, "C"),
  (0x2B04, 0x2B07, "A"),
  (0x2B0C, 0x2B11, "A"),
  (0x2B30, 0x2B3E, "A"),
  (0x2B40, 0x2B4C, "A"),
  (0x2B60, 0x2B65, "A"),
  (0x2B6A, 0x2B6D, "A"),
  (0x2B70, 0x2B73, "A"),
  (0x2B7A, 0x2B7D, "A"),
  (0x2B80, 0x2B87, "A"),
  (0x2B95, 0x2B95, "A"),
  (0x2BA0, 0x2BAF, "A"),
  (0x2BB8, 0x2BB8, "A"),
];
const PREFIX_TABLE: &[(u32, u32, &str)] = &[
  (0x0021, 0x0021, "D"),
  (0x0028, 0x0028, "F"),
  (0x002B, 0x002B, "D"),
  (0x002D, 0x002D, "D"),
  (0x005B, 0x005B, "F"),
  (0x007B, 0x007B, "F"),
  (0x007C, 0x007C, "F"),
  (0x00AC, 0x00AC, "D"),
  (0x00B1, 0x00B1, "D"),
  (0x0331, 0x0331, "D"),
  (0x2016, 0x2016, "F"),
  (0x2018, 0x2018, "D"),
  (0x201C, 0x201C, "D"),
  (0x2145, 0x2146, "L"),
  (0x2200, 0x2201, "D"),
  (0x2202, 0x2202, "L"),
  (0x2203, 0x2204, "D"),
  (0x2207, 0x2207, "D"),
  (0x220F, 0x2211, "J"),
  (0x2212, 0x2213, "D"),
  (0x221A, 0x221C, "L"),
  (0x221F, 0x2222, "D"),
  (0x222B, 0x2233, "H"),
  (0x2234, 0x2235, "D"),
  (0x223C, 0x223C, "D"),
  (0x22BE, 0x22BF, "D"),
  (0x22C0, 0x22C3, "J"),
  (0x2308, 0x2308, "F"),
  (0x230A, 0x230A, "F"),
  (0x2310, 0x2310, "D"),
  (0x2319, 0x2319, "D"),
  (0x2329, 0x2329, "F"),
  (0x2772, 0x2772, "F"),
  (0x2795, 0x2796, "D"),
  (0x27C0, 0x27C0, "D"),
  (0x27E6, 0x27E6, "F"),
  (0x27E8, 0x27E8, "F"),
  (0x27EA, 0x27EA, "F"),
  (0x27EC, 0x27EC, "F"),
  (0x27EE, 0x27EE, "F"),
  (0x2980, 0x2980, "F"),
  (0x2983, 0x2983, "F"),
  (0x2985, 0x2985, "F"),
  (0x2987, 0x2987, "F"),
  (0x2989, 0x2989, "F"),
  (0x298B, 0x298B, "F"),
  (0x298D, 0x298D, "F"),
  (0x298F, 0x298F, "F"),
  (0x2991, 0x2991, "F"),
  (0x2993, 0x2993, "F"),
  (0x2995, 0x2995, "F"),
  (0x2997, 0x2997, "F"),
  (0x2999, 0x2999, "F"),
  (0x299B, 0x29AF, "D"),
  (0x29D8, 0x29D8, "F"),
  (0x29DA, 0x29DA, "F"),
  (0x29FC, 0x29FC, "F"),
  (0x2A00, 0x2A0A, "J"),
  (0x2A0B, 0x2A1C, "H"),
  (0x2A1D, 0x2A1E, "J"),
  (0x2AEC, 0x2AED, "D"),
  (0x2AFC, 0x2AFC, "J"),
  (0x2AFF, 0x2AFF, "J"),
];
const POSTFIX_TABLE: &[(u32, u32, &str)] = &[
  (0x0021, 0x0022, "E"),
  (0x0025, 0x0027, "E"),
  (0x0029, 0x0029, "G"),
  (0x005D, 0x005D, "G"),
  (0x005E, 0x005F, "I"),
  (0x0060, 0x0060, "E"),
  (0x007C, 0x007C, "G"),
  (0x007D, 0x007D, "G"),
  (0x007E, 0x007E, "I"),
  (0x00A8, 0x00A8, "E"),
  (0x00AF, 0x00AF, "I"),
  (0x00B0, 0x00B0, "E"),
  (0x00B2, 0x00B4, "E"),
  (0x00B8, 0x00B9, "E"),
  (0x02C6, 0x02C7, "I"),
  (0x02C9, 0x02C9, "I"),
  (0x02CA, 0x02CB, "E"),
  (0x02CD, 0x02CD, "I"),
  (0x02D8, 0x02DA, "E"),
  (0x02DC, 0x02DC, "I"),
  (0x02DD, 0x02DD, "E"),
  (0x02F7, 0x02F7, "I"),
  (0x0302, 0x0302, "I"),
  (0x0311, 0x0311, "E"),
  (0x0320, 0x0320, "E"),
  (0x0325, 0x0325, "E"),
  (0x0327, 0x0327, "E"),
  (0x0331, 0x0331, "E"),
  (0x2016, 0x2016, "G"),
  (0x2019, 0x201B, "E"),
  (0x201D, 0x201F, "E"),
  (0x2032, 0x2037, "E"),
  (0x203E, 0x203E, "I"),
  (0x2057, 0x2057, "E"),
  (0x20DB, 0x20DC, "E"),
  (0x2309, 0x2309, "G"),
  (0x230B, 0x230B, "G"),
  (0x2322, 0x2323, "I"),
  (0x232A, 0x232A, "G"),
  (0x23B4, 0x23B5, "I"),
  (0x23CD, 0x23CD, "E"),
  (0x23DC, 0x23E1, "I"),
  (0x2773, 0x2773, "G"),
  (0x27E7, 0x27E7, "G"),
  (0x27E9, 0x27E9, "G"),
  (0x27EB, 0x27EB, "G"),
  (0x27ED, 0x27ED, "G"),
  (0x27EF, 0x27EF, "G"),
  (0x2980, 0x2980, "G"),
  (0x2984, 0x2984, "G"),
  (0x2986, 0x2986, "G"),
  (0x2988, 0x2988, "G"),
  (0x298A, 0x298A, "G"),
  (0x298C, 0x298C, "G"),
  (0x298E, 0x298E, "G"),
  (0x2990, 0x2990, "G"),
  (0x2992, 0x2992, "G"),
  (0x2994, 0x2994, "G"),
  (0x2996, 0x2996, "G"),
  (0x2998, 0x2998, "G"),
  (0x2999, 0x2999, "G"),
  (0x29D9, 0x29D9, "G"),
  (0x29DB, 0x29DB, "G"),
  (0x29FD, 0x29FD, "G"),
  (0x1EEF0, 0x1EEF1, "I"),
];
const FENCE_TABLE: &[(u32, u32)] = &[
  (0x0028, 0x0029),
  (0x005B, 0x005B),
  (0x005D, 0x005D),
  (0x007B, 0x007D),
  (0x0331, 0x0331),
  (0x2016, 0x2016),
  (0x2018, 0x2019),
  (0x201C, 0x201D),
  (0x2308, 0x230B),
  (0x2329, 0x232A),
  (0x2772, 0x2773),
  (0x27E6, 0x27EF),
  (0x2980, 0x2980),
  (0x2983, 0x2999),
  (0x29D8, 0x29DB),
  (0x29FC, 0x29FD),
];

/// Binary-search a (start, end, category) range table.
fn range_lookup(table: &[(u32, u32, &'static str)], code: u32) -> Option<&'static str> {
  match table.binary_search_by(|&(a, b, _)| {
    if code < a {
      std::cmp::Ordering::Greater
    } else if code > b {
      std::cmp::Ordering::Less
    } else {
      std::cmp::Ordering::Equal
    }
  }) {
    Ok(i) => Some(table[i].2),
    Err(_) => None,
  }
}

fn lookup_infix(code: u32) -> Option<&'static str> { range_lookup(INFIX_TABLE, code) }
fn lookup_prefix(code: u32) -> Option<&'static str> { range_lookup(PREFIX_TABLE, code) }
fn lookup_postfix(code: u32) -> Option<&'static str> { range_lookup(POSTFIX_TABLE, code) }

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
  FENCE_TABLE.iter().any(|&(a, b)| (a..=b).contains(&c))
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
  fn test_content_form_table_regressions() {
    // U+2A50 (⩐) was misclassified Cat B (MED) — Perl: Cat C (THIN).
    let p = opdict_lookup("\u{2A50}", "BINOP");
    assert_eq!(p.lspace, THIN);
    assert_eq!(p.rspace, THIN);
    // U+27A1 (➡) sat in a Cat-A dingbat-arrow hole — Perl: Cat A (THICK).
    let p = opdict_lookup("\u{27A1}", "ARROW");
    assert_eq!(p.lspace, THICK);
    // U+0331 combining macron-below is in the fence table (Perl).
    let p = opdict_lookup("\u{0331}", "OPEN");
    assert!(p.fence);
  }

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
