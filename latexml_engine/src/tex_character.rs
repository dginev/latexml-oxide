//! TeX Character
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::compose;

static SPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s").unwrap());

/// Convert a char-code register argument (`\catcode`, `\lccode`, `\uccode`,
/// `\sfcode`) to the `char` it keys. LaTeXML is Unicode-aware: a code like
/// `\catcode`‹=\active` carries the FULL codepoint (U+2039 = 8249), so the
/// old `(n as u8) as char` truncated it to `8249 & 0xFF = 57` ('9') —
/// silently activating the wrong character (witness: csquotes
/// `\MakeAutoQuote*{‹}{›}` made '9'/':' active+undefined, 2007.09691). Perl
/// keys the catcode table on `chr($charcode)` with no truncation. Mirror that
/// with `char::from_u32`; only fall back to the 8-bit form for an out-of-range
/// code (negative / surrogate / > U+10FFFF), where no valid `char` exists.
#[inline]
fn charcode_to_char(n: i64) -> char {
  char::from_u32(n as u32).unwrap_or((n as u8) as char)
}

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Character Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // \ (ctrl space)    c  inserts a control space.
  // \char           c  provides access to one of the 256 characters in a font.
  //----------------------------------------------------------------------
  // Perl: $_[0]->enterHorizontal; Box(' ', ...isSpace => 1, width => '0.5em')
  DefPrimitive!("\\ ", {
    enter_horizontal();
    Tbox::new(
      arena::pin_static(" "),
      None,
      None,
      Tokens!(T_CS!("\\ ")),
      stored_map!("name" => "space", "isSpace" => true,
      "width" => Dimension::from_str("0.5em")?),
    )
  });

  // Perl: $stomach->enterHorizontal; Box($glyph, $adjfont, ...)
  DefPrimitive!("\\char Number", sub[(number)] {
    enter_horizontal();
    let number_tks = number.revert().unwrap_or_default().unlist();
    let decoded = match font::decode_str(number.value_of() as u8, None, false) {
      None => pin!(""),
      Some(s) => s
    };
    Tbox::new(
     decoded,
     None,
     None,
     Tokens!(T_CS!("\\char"), number_tks, T_RELAX!()),
     SymHashMap::default())
  });

  // No `mode => "text"`: Perl's `DefAccent` Primitive (TeX_Character.pool.ltxml
  // L92-100) does NOT force text mode either. Forcing text mode here breaks
  // user `\def\k#1{|#1\rangle}` (and similar math-shorthand redefinitions of
  // accent CSes) that get clobbered by the kernel `DefAccent('\k',...)` at
  // pool-load time. After the clobber, `\k{\phi_i}` invokes the accent;
  // forcing text mode caused `_` in the arg's digestion to fire
  // "Script _ can only appear in math mode". `apply_accent` calls
  // `stomach::digest(letter)` which now uses the call-site mode (math here),
  // so `\phi_i` digests as math subscript — matching Perl. Witness:
  // quant-ph0109041 (R=67 → expected 0).
  DefPrimitive!("\\lx@applyaccent DefToken Token Token {}",
  sub[(accent, combiningchar, standalonechar, letter)] {
    let combiningchar = combiningchar.with_str(|s| s.chars().next()).unwrap();
    let standalonechar = standalonechar.to_string();
    apply_accent(letter.clone(), combiningchar, &standalonechar, Some(
      Tokens!(T_CS!(accent.to_string()),T_BEGIN!(),letter,T_END!())))
  });

  // # This will fail if there really are "assignments" after the number!
  // # We're given a number pointing into the font, from which we can derive the standalone char.
  // # From that, we want to figure out the combining character, but there could be one for
  // # both the above & below cases!  We'll prefer the above case.
  // DefPrimitive('\accent Number {}', sub {
  //     my ($stomach, $num, $letter) = @_;
  //     my $n        = $num->valueOf;
  //     my $fontinfo = lookupFontinfo(LookupValue('textfont_0'));
  //     my $acc      = ($fontinfo && $$fontinfo{encoding} ? FontDecode($n, $$fontinfo{encoding}) :
  // chr($n));     my $reversion = Invocation(T_CS('\accent'), $num, $letter);
  //     # NOTE: REVERSE LOOKUP in above accent list for the non-spacing accent char
  //     # BUT, \accent always (?) makes an above type accent... doesn't it?
  //     if (my $combiner = LookupMapping('accent_combiner_above', $acc)
  //       || LookupMapping('accent_combiner_below', $acc)) {
  //       applyAccent($stomach, $letter, $combiner, $acc, $reversion); }
  //     else {
  //       Warn('unexpected', "accent$n", $stomach, "Accent '$n' not recognized");
  //       Box(ToString($letter), undef, undef, $reversion); } });

  //======================================================================
  // \chardef        iq provides an alternate way to define a control sequence that returns a
  // character.
  //----------------------------------------------------------------------

  // Almost like a register (and \countdef), but different...
  // (including the preassignment to \relax!)
  DefPrimitive!("\\chardef Token SkipSpaces SkipMatch:=", sub[(newcs)] {
    // Let w/o AfterAssignment
    let relax_meaning = lookup_meaning(&TOKEN_RELAX).unwrap();
    state::assign_meaning(&newcs, relax_meaning, None);
    let value = gullet::read_number()?;
    state::install_definition(
      Register::new_chardef(newcs, Some(value.into()), None, None), None);
    state::after_assignment();
    Ok(Vec::new())
  });

  //======================================================================
  // Upper/Lowercase
  //----------------------------------------------------------------------
  // \lowercase      c  converts tokens to lowercase.
  // \uppercase      c  converts tokens to uppercase.
  // \uppercase<general text>, \lowercase<general text>

  // Note that these are NOT expandable, even though the "return" tokens!
  DefPrimitive!("\\uppercase GeneralText", sub[(tokens)] {
    gullet::unread_vec(
      tokens.unlist().into_iter()
        .map(uppercase_token)
        .collect());
  });
  DefPrimitive!("\\lowercase GeneralText", sub[(tokens)] {
    gullet::unread_vec(
      tokens.unlist().into_iter()
        .map(lowercase_token)
        .collect::<Vec<Token>>());
  });

  //======================================================================
  // Converting things to strings (tokens, really)
  //----------------------------------------------------------------------
  // \number         c  produces the decimal equivalent of numbers.
  // \romannumeral   c  converts a number to lowercase roman numerals.
  // \string         c  converts a control sequence to characters.

  DefMacro!("\\number Number", sub[(num)] { Explode!(num.value_of()) });
  DefMacro!("\\romannumeral Number", sub[(num)] { roman!(num.value_of()) });
  // 1) Knuth, The TeXBook, page 40, paragraph 1, Chapter 7: How TEX Reads What You Type.
  // suggests all characters except spaces are returned in category code Other, i.e. Explode()
  // Mirrors Perl: CS → explode with escape char; SPACE → keep as space; ESCAPE/COMMENT/INVALID →
  // empty; all other catcodes → T_OTHER with same text.
  DefMacro!("\\string Token", sub[(token)] {
    match token.code {
      Catcode::CS => {
        let mut s = token.to_string();
        if s.starts_with('\\') {
          s = escapechar() + &s[1..];
        }
        Explode!(s)
      }
      Catcode::SPACE => vec![token],
      Catcode::ESCAPE | Catcode::COMMENT | Catcode::INVALID => vec![],
      _ => vec![Token { text: token.text, code: Catcode::OTHER, #[cfg(feature = "token-locators")] loc: 0 }],
    }
  });

  //======================================================================
  // Character properties
  //----------------------------------------------------------------------
  // \catcode        iq holds the category code for a character.
  // \lccode                 iq holds the lowercase value for a character.
  // \sfcode                 iq holds the space factor value for a character.
  // \uccode                 iq holds the uppercase value for a character.
  DefRegister!("\\catcode Number", Number::new(0),
    getter => sub[args] {
      unpack_opt!(args => num);
      let refchar = charcode_to_char(num.expect_number().value_of());
      let code = lookup_catcode(refchar).unwrap_or(Catcode::OTHER);
      Number::from(code)
    },
    setter => sub[value, scope, args] {
      unpack_opt!(args => num);
      let c_char = charcode_to_char(num.expect_number().value_of());
      let c_code : Catcode = From::from(value.value_of() as u8);
      assign_catcode(c_char, c_code, scope);
    }
  );
  DefRegister!("\\sfcode Number", Number::new(0),
  getter=> sub[args] {
  let code = lookup_sfcode(charcode_to_char(args[0].value_of()));
    Number::new(code.unwrap_or(1000) as i64)  // Perl default is 1000 for undefined sfcodes
  },
  setter => sub[value, scope, args] {
    assign_sfcode(charcode_to_char(args[0].value_of()),
      value.value_of() as u16, scope); });
  DefRegister!("\\lccode Number", Number::new(0),
  getter=> sub[args] {
    let code = lookup_lccode(charcode_to_char(args[0].value_of()));
    Number::new(code.unwrap_or_default() as i64)
  },
  setter => sub[value, scope, args] {
    assign_lccode(charcode_to_char(args[0].value_of()),
      value.value_of() as u16, scope);
  });
  DefRegister!("\\uccode Number", Number::new(0),
  getter=> sub[args] {
    let code = lookup_uccode(charcode_to_char(args[0].value_of()));
    Number::new(code.unwrap_or_default() as i64)
  },
  setter => sub[value, scope, args] {
    assign_uccode(charcode_to_char(args[0].value_of()),
      value.value_of() as u16, scope);
  });

  //======================================================================
  // Special character codes
  //----------------------------------------------------------------------
  // \endlinechar    pi is the character added to the end of input lines.
  // \escapechar     pi is the character used for category 0 characters when outputting control
  // sequences. \newlinechar    pi is the character which begins a new line of output.
  DefRegister!("\\endlinechar", Number!(13));
  DefRegister!("\\escapechar", Number!(92));
  DefRegister!("\\newlinechar", Number!(-1));
});

// Create a box applying an accent to a letter
// Hopefully, we'll get a Box from digestion with a plain string.
// Then we can apply combining accents to it.
pub fn apply_accent(
  letter: Tokens,
  combiningchar: char,
  standalonechar: &str,
  reversion: Option<Tokens>,
) -> Result<Tbox> {
  let letter_box = stomach::digest(letter)?;
  let locator = letter_box.get_locator();
  let font = letter_box.get_font()?.map(|f| Rc::new((*f).clone()));

  let mut string: String = letter_box.to_string();
  // Perl: only replace dotless i/j with dotted for over-accents.
  // Over-accents are those placed above the letter (combiner range U+0300–U+0315 approx.).
  // Below accents (cedilla U+0327, dot-below U+0323, macron-below U+0331, comma-below U+0326)
  // preserve dotless i/j so combining works correctly.
  // U+0300–U+0315 covers grave/acute/circumflex/tilde/macron/dot-above/diaeresis/ring/caron etc.
  // U+0361 (COMBINING DOUBLE INVERTED BREVE from \t) is also an above accent.
  let is_above_accent = matches!(combiningchar, '\u{0300}'..='\u{0315}' | '\u{0361}');
  if is_above_accent {
    string = string.replace('\u{0131}', "i").replace('\u{0237}', "j");
  }
  string = SPACE_RE.replace_all(&string, " ").into_owned();

  // Perl: applying combining dot above (U+0307) to i or j is redundant — remove it.
  let effective_combiner = if combiningchar == '\u{0307}' && string.contains(['i', 'j']) {
    '\0' // sentinel for "no combining char"
  } else {
    combiningchar
  };

  // HACK to mimic real LaTeX's encoding mechanism (from Perl).
  // Necessary for \~, \^ in urls, ascii, typewriter contexts.
  // In typewriter font or ASCII encoding, produce the plain character
  // instead of applying the combining accent.
  let typewriter_replacement = match standalonechar {
    "\u{02DC}" => Some("~"), // SMALL TILDE → ~
    "\u{02C6}" => Some("^"), // MODIFIER CIRCUMFLEX → ^
    _ => None,
  };
  if let Some(replacement) = typewriter_replacement {
    if let Some(ref f) = font {
      let is_typewriter = f
        .get_family()
        .is_some_and(|fam| fam.as_ref() == "typewriter");
      let is_ascii = f.get_encoding().is_some_and(|enc| enc.as_ref() == "ASCII");
      if is_typewriter || is_ascii {
        return Ok(Tbox::new(
          arena::pin(format!("{replacement}{string}")),
          font,
          locator,
          reversion.unwrap_or(Tokens!()),
          SymHashMap::default(),
        ));
      }
    }
  }

  let text = if string.chars().all(|l| l.is_whitespace()) {
    standalonechar.to_string()
  } else {
    let mut letters = string.chars();
    let lead_letter = letters.next().unwrap();
    let mut combined_str = if effective_combiner == '\0' {
      lead_letter.to_string()
    } else {
      compose(lead_letter, effective_combiner)
        .map(|c| c.to_string())
        .unwrap_or_else(|| format!("{lead_letter}{effective_combiner}"))
    };
    for rest in letters {
      combined_str.push(rest);
    }
    combined_str.nfc().collect::<String>()
  };
  Ok(Tbox::new(
    arena::pin(text),
    font,
    locator,
    reversion.unwrap_or(Tokens!()),
    SymHashMap::default(),
  ))
}

/// Accent data entry: maps a character (combiner or standalone) to its accent properties.
/// Perl: @accent_data in LaTeXML/Util/Unicode.pm
pub struct AccentEntry {
  pub combiner:   char,
  pub standalone: &'static str,
  pub unwrapped:  &'static str,
  pub name:       &'static str,
  pub role:       &'static str,
}

/// Lookup accent data by standalone or combiner character.
/// Perl: sub unicode_accent in LaTeXML/Util/Unicode.pm
pub fn unicode_accent(glyph: &str) -> Option<&'static AccentEntry> {
  // Table from Perl: @accent_data in LaTeXML/Util/Unicode.pm
  static ACCENT_DATA: &[AccentEntry] = &[
    AccentEntry {
      combiner:   '\u{0300}',
      standalone: "\u{0060}",
      unwrapped:  "`",
      name:       "grave",
      role:       "OVERACCENT",
    },
    AccentEntry {
      combiner:   '\u{0301}',
      standalone: "\u{00B4}",
      unwrapped:  "\u{00B4}",
      name:       "acute",
      role:       "OVERACCENT",
    },
    AccentEntry {
      combiner:   '\u{0302}',
      standalone: "\u{02C6}",
      unwrapped:  "\u{005E}",
      name:       "hat",
      role:       "OVERACCENT",
    },
    AccentEntry {
      combiner:   '\u{0308}',
      standalone: "\u{00A8}",
      unwrapped:  "\u{00A8}",
      name:       "ddot",
      role:       "OVERACCENT",
    },
    AccentEntry {
      combiner:   '\u{0303}',
      standalone: "\u{02DC}",
      unwrapped:  "\u{007E}",
      name:       "tilde",
      role:       "OVERACCENT",
    },
    AccentEntry {
      combiner:   '\u{0304}',
      standalone: "\u{00AF}",
      unwrapped:  "\u{00AF}",
      name:       "bar",
      role:       "OVERACCENT",
    },
    AccentEntry {
      combiner:   '\u{0307}',
      standalone: "\u{02D9}",
      unwrapped:  "\u{02D9}",
      name:       "dot",
      role:       "OVERACCENT",
    },
    AccentEntry {
      combiner:   '\u{030B}',
      standalone: "\u{02DD}",
      unwrapped:  "\u{2032}\u{2032}",
      name:       "dtick",
      role:       "OVERACCENT",
    },
    AccentEntry {
      combiner:   '\u{0306}',
      standalone: "\u{02D8}",
      unwrapped:  "\u{02D8}",
      name:       "breve",
      role:       "OVERACCENT",
    },
    AccentEntry {
      combiner:   '\u{030C}',
      standalone: "\u{02C7}",
      unwrapped:  "\u{02C7}",
      name:       "check",
      role:       "OVERACCENT",
    },
    AccentEntry {
      combiner:   '\u{030A}',
      standalone: "\u{02DA}",
      unwrapped:  "\u{02DA}",
      name:       "ring",
      role:       "OVERACCENT",
    },
    AccentEntry {
      combiner:   '\u{20D7}',
      standalone: "\u{00A0}\u{20D7}",
      unwrapped:  "\u{2192}",
      name:       "vec",
      role:       "OVERACCENT",
    },
    AccentEntry {
      combiner:   '\u{0311}',
      standalone: "\u{00A0}\u{0311}",
      unwrapped:  "u",
      name:       "arch",
      role:       "OVERACCENT",
    },
    AccentEntry {
      combiner:   '\u{0361}',
      standalone: "\u{00A0}\u{0361}",
      unwrapped:  "u",
      name:       "tie",
      role:       "OVERACCENT",
    },
    AccentEntry {
      combiner:   '\u{0327}',
      standalone: "\u{00B8}",
      unwrapped:  "\u{00B8}",
      name:       "cedilla",
      role:       "UNDERACCENT",
    },
    AccentEntry {
      combiner:   '\u{0323}',
      standalone: ".",
      unwrapped:  "\u{22C5}",
      name:       "underdot",
      role:       "UNDERACCENT",
    },
    AccentEntry {
      combiner:   '\u{0331}',
      standalone: "_",
      unwrapped:  "\u{00AF}",
      name:       "underbar",
      role:       "UNDERACCENT",
    },
    AccentEntry {
      combiner:   '\u{0326}',
      standalone: ",",
      unwrapped:  ",",
      name:       "lfhook",
      role:       "UNDERACCENT",
    },
    AccentEntry {
      combiner:   '\u{0328}',
      standalone: "\u{02DB}",
      unwrapped:  "\u{02DB}",
      name:       "ogonek",
      role:       "UNDERACCENT",
    },
  ];

  for entry in ACCENT_DATA {
    if entry.standalone == glyph {
      return Some(entry);
    }
    if glyph.len() == entry.combiner.len_utf8() && glyph.starts_with(entry.combiner) {
      return Some(entry);
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accent_by_standalone_grave_backtick() {
    // U+0060 `GRAVE ACCENT` (standalone) matches the grave entry.
    let entry = unicode_accent("`").expect("grave accent");
    assert_eq!(entry.name, "grave");
    assert_eq!(entry.combiner, '\u{0300}');
    assert_eq!(entry.role, "OVERACCENT");
  }

  #[test]
  fn accent_by_combiner_grave() {
    // Lone combining grave U+0300 is recognized via the combiner field.
    let entry = unicode_accent("\u{0300}").expect("combining grave");
    assert_eq!(entry.name, "grave");
  }

  #[test]
  fn accent_by_standalone_acute() {
    let entry = unicode_accent("\u{00B4}").expect("acute standalone");
    assert_eq!(entry.name, "acute");
    assert_eq!(entry.role, "OVERACCENT");
  }

  #[test]
  fn accent_hat_circumflex() {
    let entry = unicode_accent("\u{02C6}").expect("modifier letter circumflex");
    assert_eq!(entry.name, "hat");
  }

  #[test]
  fn accent_below_role_is_underaccent() {
    // Cedilla is a below-accent, mapped to UNDERACCENT role.
    let cedilla = unicode_accent("\u{00B8}").expect("cedilla");
    assert_eq!(cedilla.name, "cedilla");
    assert_eq!(cedilla.role, "UNDERACCENT");
    // Underbar '_' ditto.
    let underbar = unicode_accent("_").expect("underbar");
    assert_eq!(underbar.name, "underbar");
    assert_eq!(underbar.role, "UNDERACCENT");
  }

  #[test]
  fn accent_none_for_unknown_glyph() {
    assert!(unicode_accent("x").is_none());
    assert!(unicode_accent("").is_none());
    assert!(unicode_accent("AB").is_none());
  }

  #[test]
  fn accent_unwrapped_field_present() {
    // Regression for the dtick entry which has a 2-codepoint unwrapped value.
    let entry = unicode_accent("\u{02DD}").expect("double acute");
    assert_eq!(entry.name, "dtick");
    assert_eq!(entry.unwrapped, "\u{2032}\u{2032}");
  }
}
