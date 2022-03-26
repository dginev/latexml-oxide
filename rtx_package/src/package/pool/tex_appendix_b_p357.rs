use crate::package::*;

LoadDefinitions!(state, {
  //// NOTE that a 3rd form seems desirable: an concise form that cannot rely on context for the
  //// type. This would be useful for the titles in links; thus can be plain (unicode) text.

  //======================================================================
  // TeX Book, Appendix B. p. 356

  DefMacro!("\\raggedright", "");
  DefMacro!("\\raggedleft", ""); // this is actually LaTeX
  DefMacro!("\\ttraggedright", "");
  DefMacro!("\\leavevmode", "");

  //----------------------------------------------------------------------
  // Actually from LaTeX; Table 3.2. Non-English Symbols, p.39

  // The following shouldn't appear in math.
  DefMacro!("\\OE", "\u{0152}"); // LATIN CAPITAL LIGATURE OE
  DefMacro!("\\oe", "\u{0153}"); // LATIN SMALL LIGATURE OE
  DefMacro!("\\AE", "\u{00C6}"); // LATIN CAPITAL LETTER AE
  DefMacro!("\\ae", "\u{00E6}"); // LATIN SMALL LETTER AE
  DefMacro!("\\AA", "\u{00C5}"); // LATIN CAPITAL LETTER A WITH RING ABOVE
  DefMacro!("\\aa", "\u{00E5}"); // LATIN SMALL LETTER A WITH RING ABOVE
  DefMacro!("\\O", "\u{00D8}"); // LATIN CAPITAL LETTER O WITH STROKE
  DefMacro!("\\o", "\u{00F8}"); // LATIN SMALL LETTER O WITH STROKE
  DefMacro!("\\L", "\u{0141}"); // LATIN CAPITAL LETTER L WITH STROKE
  DefMacro!("\\l", "\u{0142}"); // LATIN SMALL LETTER L WITH STROKE
  DefMacro!("\\ss", "\u{00DF}"); // LATIN SMALL LETTER SHARP S

  // apparently the rest can appear in math.
  DefMacro!("\\lx@sectionsign", "\u{00a7}"); // SECTION SIGN
  DefMacro!("\\lx@paragraphsign", "\u{00B6}"); // PILCROW SIGN
  DefMacro!("\\S", "\\lx@sectionsign");
  DefMacro!("\\P", "\\lx@paragraphsign");
  DefMacro!("\\dag", "\u{2020}"); // DAGGER
  DefMacro!("\\ddag", "\u{2021}"); // DOUBLE DAGGER
  DefMacro!("\\copyright", "\u{00A9}"); // COPYRIGHT SIGN
  DefMacro!("\\pounds", "\u{00A3}"); // POUND SIGN

  // #----------------------------------------------------------------------
  // # Accents.  LaTeX Table 3.1, p.38
  // #----------------------------------------------------------------------
  // # All of TeX's accents can (sorta) be handled by Unicode's combining accents
  // # (which follow the character to be accented).
  // # We'll let unicode normalization do the combination, if needed.
  // # Also, note that \t is intended to combine multiple chars, but it appears to
  // # work (via mozilla !?) best when the combining char is after the 1st char.
  // # Further, the accents \d and \b seem to center the under dot or bar under multiple
  // # chars --- how should this be handled in Unicode?

  // # Since people sometimes try to get fancy by using an empty argument,
  // # for each, I'm providing the combining code and an equivalent(?) spacing one.
  // # (doesn't look quite the same to use a combining char after a space)

  // # Create a box applying an accent to a letter
  // # Hopefully, we'll get a Box from digestion with a plain string.
  // # Then we can apply combining accents to it.
  // sub applyAccent {
  //   my ($stomach, $letter, $combiningchar, $standalonechar, $reversion) = @_;
  //   my $box     = Digest($letter);
  //   my $locator = $box->getLocator;
  //   my $font    = $box->getFont;
  //   my $string  = $box->toString;
  //   $string =~ tr/\x{0131}\x{0237}/ij/;
  //   $string =~ s/\s/ /g;
  //   my @letters = split(//, $string);
  //   return Box(($string =~ /^\s*$/
  //       ? $standalonechar
  //       : NFC($letters[0] . $combiningchar . join('', @letters[1 .. $#letters]))),
  //     $font, $locator, $reversion); }

  // # Defines an accent command using a combining char that follows the
  // # 1st char of the argument.  In cases where there is no argument, $standalonechar is used.
  // sub DefAccent {
  //   my ($accent, $combiningchar, $standalonechar, %options) = @_;
  //   $options{above} = 1 if !(defined $options{above}) && !$options{below};
  //   # Used for converting a char used as an above-accent to a combining char (See \accent)
  //   AssignMapping('accent_combiner_above', $standalonechar => $combiningchar) if $options{above};
  //   AssignMapping('accent_combiner_below', $standalonechar => $combiningchar) unless
  // $options{above};   DefPrimitive($accent . "{}", sub {
  //       my ($stomach, $letter) = @_;
  //       applyAccent($stomach, $letter, $combiningchar, $standalonechar, Invocation($accent,
  // $letter)); },     mode => 'text');
  //   return; }

  // DefAccent('\`',           "\x{0300}", UTF(0x60));  # COMBINING GRAVE ACCENT & GRAVE ACCENT
  // DefAccent("\\'",          "\x{0301}", UTF(0xB4));  # COMBINING ACUTE ACCENT & ACUTE ACCENT
  // DefAccent('\^',           "\x{0302}", UTF(0x5E));  # COMBINING CIRCUMFLEX ACCENT & CIRCUMFLEX
  // ACCENT DefAccent('\"',           "\x{0308}", UTF(0xA8));  # COMBINING DIAERESIS & DIAERESIS
  DefAccent!("\\~", "\u{0303}", "~"); // COMBINING TILDE
                                      // DefAccent('\=',           "\x{0304}", UTF(0xAF));  # COMBINING MACRON & MACRON
                                      // DefAccent('\.',           "\x{0307}", "\x{02D9}"); # COMBINING DOT ABOVE & DOT ABOVE
                                      // DefAccent('\u',           "\x{0306}", "\x{02D8}"); # COMBINING BREVE & BREVE
                                      // DefAccent('\v',           "\x{030C}", "\x{02C7}"); # COMBINING CARON & CARON
                                      // DefAccent('\@ringaccent', "\x{030A}", "o");        # COMBINING RING ABOVE & non-combining
                                      // DefAccent('\r',           "\x{030A}", "o");        # COMBINING RING ABOVE & non-combining
                                      // DefAccent('\H',           "\x{030B}", "\x{02DD}"); # COMBINING DOUBLE ACUTE ACCENT &
                                      // non-combining DefAccent('\c', "\x{0327}", UTF(0xB8), below => 1);    # COMBINING CEDILLA &
                                      // CEDILLA       # NOTE: The next two get define for math, as well; See below
                                      // DefAccent('\@text@daccent', "\x{0323}", '.',       below => 1);   # COMBINING DOT BELOW & DOT
                                      // (?) DefAccent('\@text@baccent', "\x{0331}", UTF(0xAF), below => 1);   # COMBINING MACRON
                                      // BELOW  & MACRON DefAccent('\t', "\x{0361}", "-");    # COMBINING DOUBLE INVERTED BREVE & ????
                                      // What????       # this one's actually defined in mathscinet.sty, but just stick it here!
                                      // DefAccent('\lfhook', "\x{0326}", ",", below => 1);   # COMBINING COMMA BELOW
                                      //                                                      # I doubt that latter covers multiple
                                      // chars...?         #DefAccent('\bar',"\x{0304}", ?);  # COMBINING MACRON or is this the longer
                                      // overbar?

  // # This will fail if there really are "assignments" after the number!
  // # We're given a number pointing into the font, from which we can derive the standalone char.
  // # From that, we want to figure out the combining character, but there could be one for
  // # both the above & below cases!  We'll prefer the above case.
  // DefPrimitive('\accent Number {}', sub {
  //     my ($stomach, $num, $letter) = @_;
  //     my $n        = $num->valueOf;
  //     my $fam      = 0;                                            # ?
  //     my $font     = LookupValue('fontinfo_' . $fam . '_text');
  //     my $fontinfo = LookupValue('fontinfo_' . ToString($font));
  //     my $acc = ($fontinfo && $$fontinfo{encoding} ? FontDecode($n, $$fontinfo{encoding}) :
  // chr($n));     my $reversion = Invocation(T_CS('\accent'), $num, $letter);
  //     # NOTE: REVERSE LOOKUP in above accent list for the non-spacing accent char
  //     # BUT, \accent always (?) makes an above type accent... doesn't it?
  //     if (my $combiner = LookupMapping('accent_combiner_above', $acc)
  //       || LookupMapping('accent_combiner_below', $acc)) {
  //       applyAccent($stomach, $letter, $combiner, $acc, $reversion); }
  //     else {
  //       Warn('unexpected', "accent$n", $stomach, "Accent '$n' not recognized");
  //       Box(ToString($letter), undef, undef, $reversion); } });

  // // Note that these two apparently work in Math? BUT the argument is treated as text!!!
  // DefMacro('\d{}', '\ifmmode\@math@daccent{#1}\else\@text@daccent{#1}\fi');
  // DefMacro('\b{}', '\ifmmode\@math@baccent{#1}\else\@text@baccent{#1}\fi');

  // DefConstructor('\@math@daccent {}',
  //   "<ltx:XMApp><ltx:XMTok role='UNDERACCENT'>\x{22c5}</ltx:XMTok>"
  //     . "?#textarg(<ltx:XMText>#textarg</ltx:XMText>)(<ltx:XMArg>#matharg</ltx:XMArg>)"
  //     . "</ltx:XMApp>",
  //   mode => 'text', alias => '\d',
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
  //   mode => 'text', alias => '\b',
  //   afterDigest => sub {
  //     my ($stomach, $whatsit) = @_;
  //     my $arg = $whatsit->getArg(1);
  //     if ($arg->isMath) {
  //       $whatsit->setProperty(matharg => $arg->getBody); }
  //     else {
  //       $whatsit->setProperty(textarg => $arg); }
  //     return; });

  //======================================================================
  // TeX Book, Appendix B. p. 357

  // foreach my $op ('\hrulefill', '\dotfill', '\rightarrowfill', '\leftarrowfill',
  //   '\upbracefill', '\downbracefill') {
  //   DefPrimitive($op, undef); }

  Let!("\\bye", "\\end");

  Let!("\\sp", T_SUPER!());
  Let!("\\sb", T_SUB!());

  DefMacro!("\\,", "\\ifmmode\\@math@thinmuskip\\else\\@text@thinmuskip\\fi");
  // DefConstructor!("\\@math@thinmuskip",
  //   "<ltx:XMHint name='thinspace' width='#width'/>",
  //   alias => '\,',
  //   properties => { isSpace => 1, width => sub { LookupValue('\thinmuskip'); } });
  // DefPrimitiveI('\@text@thinmuskip', undef, "\x{2009}", alias => '\,');

  DefMacro!("\\!", "\\ifmmode\\@math@negthinmuskip\\else\\@text@negthinmuskip\\fi");
  // DefConstructor('\@math@negthinmuskip', undef,
  //   "<ltx:XMHint name='negthinspace' width='#width'/>",
  //   alias => '\!',
  //   properties => { isSpace => 1,
  //     width => sub { LookupValue('\thinmuskip')->negate; } });
  // DefPrimitiveI('\@text@negthinmuskip', undef, "", alias => '\!');

  DefMacro!("\\>", "\\ifmmode\\@math@medmuskip\\else\\@text@medmuskip\\fi");
  // DefConstructor('\@math@medmuskip', undef,
  //   "<ltx:XMHint name='medspace' width='#width'/>",
  //   alias => '\>',
  //   properties => { isSpace => 1,
  //     width => sub { LookupValue('\medmuskip'); } });
  // DefPrimitiveI('\@text@medmuskip', undef, "", alias => '\>');

  DefPrimitive!("\\;", sub[stomach, args, state] {
    Tbox::new("\u{2004}".to_string(), None, None, Tokens!(T_CS!("\\;")),
      stored_map!("name"  => "thickspace", "isSpace" => true,
      "width" => state.lookup_value("\\thickmuskip")), state)
  });


  Let!("\\:", "\\>");
  DefMacro!("\\ ", "\\ifmmode\\@math@nbspace\\else\\@text@nbspace\\fi");
  // DefConstructor('\@math@nbspace', undef,
  //   "<ltx:XMHint name='medspace' width='#width'/>",
  //   alias => '\ ',
  //   properties => { isSpace => 1,
  //     width => sub { Dimension('0.5em'); } });
  DefMacro!(T_CS!("\\@text@nbspace"), None, T_OTHER!("\u{00A0}"), alias => "\\ ");

  DefMacro!("\\\t", "\\ifmmode\\@math@tab\\else\\@text@tab\\fi");
  // DefConstructor('\@math@tab', undef,    # Tab!!
  //   "<ltx:XMHint name='medspace' width='#width'/>",
  //   alias => "\\\t",                      # TAB
  //   properties => { isSpace => 1,
  //     width => sub { Dimension('1em'); } });
  // DefPrimitiveI('\@text@tab', undef, UTF(0xA0), alias => "\\\t");    # TAB!!! What else?

  DefMacro!("\\/", "\\ifmmode\\@math@italiccorr\\else\\@text@italiccorr\\fi");
  // DefConstructor("\@math@italiccorr", undef,
  //   "<ltx:XMHint name='italiccorr'/>",
  //   alias => '\/',
  //   properties => { isSpace => 1 });
  // DefPrimitiveI('\@text@italiccorr', undef, "", alias => '\/');

  // // What kind of magic might allow \mskip to translate these back into the above?
  // DefRegister!("\\thinmuskip"  , MuGlue::new("3mu"));
  // DefRegister!("\\medmuskip"   , MuGlue::new("4mu plus 2mu minus 4mu"));
  // DefRegister!("\\thickmuskip" , MuGlue::new("5mu plus 5mu"));
});
