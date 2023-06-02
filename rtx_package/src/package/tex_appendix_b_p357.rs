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
  DefMacro!(
    r"\mathhexbox{}{}{}",
    r###"\leavevmode\hbox{$\m@th \mathchar"#1#2#3$}"###
  );
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

  DefPrimitive!("\\hrulefill", None);
  DefPrimitive!("\\dotfill", None);
  DefPrimitive!("\\rightarrowfill", None);
  DefPrimitive!("\\leftarrowfill", None);
  DefPrimitive!("\\upbracefill", None);
  DefPrimitive!("\\downbracefil", None);

  Let!("\\bye", "\\end");

  Let!("\\sp", T_SUPER!());
  Let!("\\sb", T_SUB!());

  DefMacro!(
    "\\,",
    r"\ifmmode\lx@thinmuskip\else\lx@thinspace\fi",
    protected => true
  );
  // DefConstructor!("\\@math@thinmuskip",
  //   "<ltx:XMHint name='thinspace' width='#width'/>",
  //   alias => '\,',
  //   properties => { isSpace => 1, width => sub { LookupValue('\thinmuskip'); } });
  // DefPrimitiveI('\@text@thinmuskip', undef, "\x{2009}", alias => '\,');

  DefMacro!(
    "\\!",
    "\\ifmmode\\@math@negthinmuskip\\else\\@text@negthinmuskip\\fi"
  );
  // DefConstructor('\@math@negthinmuskip', undef,
  //   "<ltx:XMHint name='negthinspace' width='#width'/>",
  //   alias => '\!',
  //   properties => { isSpace => 1,
  //     width => sub { LookupValue('\thinmuskip')->negate; } });
  // DefPrimitiveI('\@text@negthinmuskip', undef, "", alias => '\!');

  DefMacro!(
    "\\>",
    "\\ifmmode\\@math@medmuskip\\else\\@text@medmuskip\\fi"
  );
  // DefConstructor('\@math@medmuskip', undef,
  //   "<ltx:XMHint name='medspace' width='#width'/>",
  //   alias => '\>',
  //   properties => { isSpace => 1,
  //     width => sub { LookupValue('\medmuskip'); } });
  // DefPrimitiveI('\@text@medmuskip', undef, "", alias => '\>');

  DefPrimitive!("\\;", sub[stomach, (), state] {
    Tbox::new(arena::pin_static("\u{2004}"), None, None, Tokens!(T_CS!("\\;")),
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

  DefMacro!(
    "\\/",
    "\\ifmmode\\@math@italiccorr\\else\\@text@italiccorr\\fi"
  );
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
