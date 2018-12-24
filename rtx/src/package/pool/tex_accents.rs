use crate::package::*;

// Create a box applying an accent to a letter
// Hopefully, we'll get a Box from digestion with a plain string.
// Then we can apply combining accents to it.
pub fn apply_accent(
  stomach: &mut Stomach,
  letter: &str,
  combiningchar: &str,
  standalonechar: &str,
  reversion: Option<Tokens>,
  state: &mut State,
) -> Result<Tbox>
{
  let letter_box = stomach.digest(TokenizeInternal!(letter, state), state)?;
  let locator = letter_box.get_locator();
  let font = match letter_box.get_font() {
    Some(f) => Some(Rc::new((*f).clone())),
    None => None,
  };

  let string = letter_box.to_string();
  // TODO:
  // string =~ tr/\u{0131}\u{0237}/ij/;
  // string =~ s/\s/ /g;
  // let letters = string.split();
  // return TBox::new(($string =~ /^\s*$/
  //     ? $standalonechar
  //     : NFC($letters[0] . $combiningchar . join('', @letters[1 .. $//letters]))),
  //   $font, $locator, $reversion); }
  Ok(Tbox::new(
    string,
    font,
    locator,
    reversion.unwrap_or(Tokens!()),
    HashMap::new(),
    state,
  ))
}

pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  //----------------------------------------------------------------------
  // Accents.  LaTeX Table 3.1, p.38
  //----------------------------------------------------------------------
  // All of TeX's accents can (sorta) be handled by Unicode's combining accents
  // (which follow the character to be accented).
  // We'll let unicode normalization do the combination, if needed.
  // Also, note that \t is intended to combine multiple chars, but it appears to
  // work (via mozilla !?) best when the combining char is after the 1st char.
  // Further, the accents \d and \b seem to center the under dot or bar under multiple
  // chars --- how should this be handled in Unicode?

  // Since people sometimes try to get fancy by using an empty argument,
  // for each, I'm providing the combining code and an equivalent(?) spacing one.
  // (doesn't look quite the same to use a combining char after a space)

  DefAccent!("\\`", "\u{0300}", "\u{0060}"); // COMBINING GRAVE ACCENT & GRAVE ACCENT

  DefAccent!("\\'", "\u{0301}", "\u{00B4}"); // COMBINING ACUTE ACCENT & ACUTE ACCENT
  DefAccent!("\\^", "\u{0302}", "\u{005E}"); // COMBINING CIRCUMFLEX ACCENT & CIRCUMFLEX ACCENT
  DefAccent!("\\\"", "\u{0308}", "\u{00A8}"); // COMBINING DIAERESIS & DIAERESIS
  DefAccent!("\\~", "\u{0303}", "~"); // COMBINING TILDE
  DefAccent!("\\=", "\u{0304}", "\u{00AF}"); // COMBINING MACRON & MACRON
  DefAccent!("\\.", "\u{0307}", "\u{02D9}"); // COMBINING DOT ABOVE & DOT ABOVE
  DefAccent!("\\u", "\u{0306}", "\u{02D8}"); // COMBINING BREVE & BREVE
  DefAccent!("\\v", "\u{030C}", "\u{02C7}"); // COMBINING CARON & CARON
  DefAccent!("\\@ringaccent", "\u{030A}", "o"); // COMBINING RING ABOVE & non-combining
  DefAccent!("\\r", "\u{030A}", "o"); // COMBINING RING ABOVE & non-combining
  DefAccent!("\\H", "\u{030B}", "\u{02DD}"); // COMBINING DOUBLE ACUTE ACCENT & non-combining
  DefAccent!("\\c", "\u{0327}", "\u{00B8}", below => true); // COMBINING CEDILLA & CEDILLA
                                                            // NOTE: The next two get define for math, as well; See below
  DefAccent!("\\@text@daccent", "\u{0323}", ".",       below => true); // COMBINING DOT BELOW & DOT (?)
  DefAccent!("\\@text@baccent", "\u{0331}", "\u{00AF}", below => true); // COMBINING MACRON BELOW  & MACRON
  DefAccent!("\\t", "\u{0361}", "-"); // COMBINING DOUBLE INVERTED BREVE & ???? What????
                                      // this one"s actually defined in mathscinet.sty, but just stick it here!
  DefAccent!("\\lfhook", "\u{0326}", ",", below => true); // COMBINING COMMA BELOW
                                                          // I doubt that latter covers multiple chars...? DefAccent("\\bar","\u{0304}", ?);  // COMBINING
                                                          // MACRON or is this the longer overbar?

  // This will fail if there really are "assignments" after the number!
  // We're given a number pointing into the font, from which we can derive the standalone char.
  // From that, we want to figure out the combining character, but there could be one for
  // both the above & below cases!  We'll prefer the above case.

  Ok(())
}
