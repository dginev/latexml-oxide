use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\SetUnicodeOption{}", "");
  DefMacro!("\\unicodevirtual{}", "#1");
  DefMacro!("\\unicodecombine", "");

  // Perl ucs.sty.ltxml L17-29: hex code → UTF-8 char. Emits a single
  // T_OTHER token carrying the decoded character. Previously stubbed
  // to empty, silently swallowing every \unichar{XXXX}.
  DefMacro!("\\unichar Expanded", sub[(hexcode)] {
    let char_str = hexcode.to_string();
    if char_str.chars().all(|c| c.is_ascii_hexdigit()) {
      if let Ok(cp) = u32::from_str_radix(&char_str, 16) {
        if cp <= 0x10FFFF {
          if let Some(ch) = char::from_u32(cp) {
            return Ok(Tokens::new(vec![T_OTHER!(ch.to_string())]));
          }
        }
        Error!("unexpected", &char_str,
          format!("{} too large for Unicode. Values between 0 and 10FFFF are permitted.",
                  char_str));
      }
    } else {
      Error!("unexpected", &char_str,
        format!("'{}' is not a hexadecimal number.", char_str));
    }
    Ok(Tokens::new(Vec::new()))
  });

  // Perl L32-40: \DeclareUnicodeCharacterAsOptional forwards to \DeclareUnicodeCharacter
  // (dropping the second arg — the "optional alternative" rendering which we don't use).
  DefMacro!(
    "\\DeclareUnicodeCharacterAsOptional{}{}{}",
    "\\DeclareUnicodeCharacter{#1}{#3}"
  );
  // Perl L33-40: shadow \DeclareUnicodeCharacter with a hex-strip pass —
  // trims the optional leading `"` before calling the saved original.
  Let!("\\@saved@DeclareUnicodeCharacter", "\\DeclareUnicodeCharacter");
  DefMacro!("\\DeclareUnicodeCharacter Expanded {}", sub[(hexcode, expansion)] {
    let mut hex = hexcode.to_string();
    if hex.starts_with('"') { hex.remove(0); }
    let mut out: Vec<Token> = vec![
      T_CS!("\\@saved@DeclareUnicodeCharacter"),
      T_OTHER!(hex),
      T_BEGIN!(),
    ];
    out.extend(expansion.unlist());
    out.push(T_END!());
    Ok(Tokens::new(out))
  });

  DefMacro!("\\DeclareUnicodeOption[]{}", "");
  DefMacro!("\\LinkUnicodeOptionToPkg{}{}", "");
  DefMacro!("\\PreloadUnicodePage{}", "");
  DefMacro!("\\PrerenderUnicode{}", "");
});
