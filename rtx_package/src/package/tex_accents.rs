use crate::package::*;
use unicode_normalization::char::compose;
use unicode_normalization::UnicodeNormalization;

static SPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s").unwrap());

// Create a box applying an accent to a letter
// Hopefully, we'll get a Box from digestion with a plain string.
// Then we can apply combining accents to it.
pub fn apply_accent(
  stomach: &mut Stomach,
  letter: Tokens,
  combiningchar: char,
  standalonechar: &str,
  reversion: Option<Tokens>,
  state: &mut State,
) -> Result<Tbox> {
  let letter_box = stomach.digest(letter, state)?;
  let locator = letter_box.get_locator();
  let font = letter_box.get_font(state)?.map(|f| Rc::new((*f).clone()));

  let mut string: String = letter_box.to_string();
  string = string.replace('\u{0131}', "i").replace('\u{0237}', "j");
  string = SPACE_RE.replace_all(&string, " ").into_owned();
  let text = if string.chars().all(|l| l.is_whitespace()) {
    standalonechar.to_string()
  } else {
    let mut letters = string.chars();
    let lead_letter = letters.next().unwrap();
    let mut combined_str = compose(lead_letter, combiningchar)
      .map(|c| c.to_string())
      .unwrap_or_else(|| format!("{lead_letter}{combiningchar}"));
    for rest in letters {
      combined_str.push(rest);
    }
    combined_str.nfc().collect::<String>()
  };
  Ok(Tbox::new(
    arena::pin(text),
    font,
    locator.map(|l| l.into_owned()),
    reversion.unwrap_or(Tokens!()),
    HashMap::default(),
    state,
  ))
}

LoadDefinitions!(state, {
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

  DefPrimitive!("\\lx@applyaccent DefToken Token Token {}",
  sub[stomach,(accent, combiningchar, standalonechar, letter),inner_state] {
    let letter_str = letter.to_string();
    let combiningchar = combiningchar.to_string().chars().next().unwrap();
    let standalonechar = standalonechar.to_string();
    apply_accent(stomach, letter.clone(), combiningchar, &standalonechar, Some(
      Tokens!(T_CS!(accent.to_string()),T_BEGIN!(),letter,T_END!())), inner_state)
  }, mode => "text");

  // Since people sometimes try to get fancy by using an empty argument,
  // for each, I'm providing the combining code and an equivalent(?) spacing one.
  // (doesn't look quite the same to use a combining char after a space)

  DefAccent!("\\`", '\u{0300}', "\u{0060}"); // COMBINING GRAVE ACCENT & GRAVE ACCENT

  DefAccent!("\\'", '\u{0301}', "\u{00B4}"); // COMBINING ACUTE ACCENT & ACUTE ACCENT
  DefAccent!("\\^", '\u{0302}', "\u{005E}"); // COMBINING CIRCUMFLEX ACCENT & CIRCUMFLEX ACCENT
  DefAccent!("\\\"", '\u{0308}', "\u{00A8}"); // COMBINING DIAERESIS & DIAERESIS
  DefAccent!("\\~", '\u{0303}', "~"); // COMBINING TILDE
  DefAccent!("\\=", '\u{0304}', "\u{00AF}"); // COMBINING MACRON & MACRON
  DefAccent!("\\.", '\u{0307}', "\u{02D9}"); // COMBINING DOT ABOVE & DOT ABOVE
  DefAccent!("\\u", '\u{0306}', "\u{02D8}"); // COMBINING BREVE & BREVE
  DefAccent!("\\v", '\u{030C}', "\u{02C7}"); // COMBINING CARON & CARON
  DefAccent!("\\@ringaccent", '\u{030A}', "o"); // COMBINING RING ABOVE & non-combining
  DefAccent!("\\r", '\u{030A}', "o"); // COMBINING RING ABOVE & non-combining
  DefAccent!("\\H", '\u{030B}', "\u{02DD}"); // COMBINING DOUBLE ACUTE ACCENT & non-combining
  DefAccent!("\\c", '\u{0327}', "\u{00B8}", below => true); // COMBINING CEDILLA & CEDILLA
  // NOTE: The next two get define for math, as well; See below
  DefAccent!("\\@text@daccent", '\u{0323}', ".",       below => true); // COMBINING DOT BELOW & DOT (?)
  DefAccent!("\\@text@baccent", '\u{0331}', "\u{00AF}", below => true); // COMBINING MACRON BELOW  & MACRON
  // COMBINING DOUBLE INVERTED BREVE & ???? What????
  DefAccent!("\\t", '\u{0361}', "-");
  // this one"s actually defined in mathscinet.sty, but just stick it here!
  // COMBINING COMMA BELOW
  DefAccent!("\\lfhook", '\u{0326}', ",", below => true);

  // This will fail if there really are "assignments" after the number!
  // We're given a number pointing into the font, from which we can derive the standalone char.
  // From that, we want to figure out the combining character, but there could be one for
  // both the above & below cases!  We'll prefer the above case.
  DefPrimitive!("\\accent Number Expanded", sub[stomach,(num,letter),state] {
    unimplemented!();
    // my $n        = $num->valueOf;
    // my $fam      = 0;                                            # ?
    // my $font     = LookupValue('fontinfo_' . $fam . '_text');
    // my $fontinfo = LookupValue('fontinfo_' . ToString($font));
    // my $acc      = ($fontinfo && $$fontinfo{encoding} ? FontDecode($n, $$fontinfo{encoding}) : chr($n));
    // my $reversion = Invocation(T_CS('\accent'), $num, $letter);
    // # NOTE: REVERSE LOOKUP in above accent list for the non-spacing accent char
    // # BUT, \accent always (?) makes an above type accent... doesn't it?
    // if (my $combiner = LookupMapping('accent_combiner_above', $acc)
    //   || LookupMapping('accent_combiner_below', $acc)) {
    //   applyAccent($stomach, $letter, $combiner, $acc, $reversion); }
    // else {
    //   Warn('unexpected', "accent$n", $stomach, "Accent '$n' not recognized");
    //   Box(ToString($letter), undef, undef, $reversion); }
    Ok(())
  });
  // Note that these two apparently work in Math? BUT the argument is treated as text!!!
  DefMacro!("\\d{}", r"\ifmmode\@math@daccent{#1}\else\@text@daccent{#1}\fi");
  DefMacro!("\\b{}", r"\ifmmode\@math@baccent{#1}\else\@text@baccent{#1}\fi");

  //   DefConstructor('\@math@daccent {}',
  //   "<ltx:XMApp><ltx:XMTok role='UNDERACCENT'>\x{22c5}</ltx:XMTok>"
  //     . "?#textarg(<ltx:XMText>#textarg</ltx:XMText>)(<ltx:XMArg>#matharg</ltx:XMArg>)"
  //     . "</ltx:XMApp>",
  //   mode        => 'text', alias => '\d',
  //   afterDigest => sub {
  //     my ($stomach, $whatsit) = @_;
  //     my $arg = $whatsit->getArg(1);
  //     if ($arg->isMath) {
  //       $whatsit->setProperty(matharg => $arg->getBody); }
  //     else {
  //       $whatsit->setProperty(textarg => $arg); }
  //     return; });

  // DefConstructor('\@math@baccent {}',
  //   "<ltx:XMApp><ltx:XMTok role='UNDERACCENT'>" . UTF(0xAF) . "</ltx:XMTok>"
  //     . "?#textarg(<ltx:XMText>#textarg</ltx:XMText>)(<ltx:XMArg>#matharg</ltx:XMArg>)"
  //     . "</ltx:XMApp>",
  //   mode        => 'text', alias => '\b',
  //   afterDigest => sub {
  //     my ($stomach, $whatsit) = @_;
  //     my $arg = $whatsit->getArg(1);
  //     if ($arg->isMath) {
  //       $whatsit->setProperty(matharg => $arg->getBody); }
  //     else {
  //       $whatsit->setProperty(textarg => $arg); }
  //     return; });


});
