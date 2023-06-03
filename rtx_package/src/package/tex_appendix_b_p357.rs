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

  // Note that these two apparently work in Math? BUT the argument is treated as text!!!
  DefMacro!("\\d{}", r"\ifmmode\@math@daccent{#1}\else\@text@daccent{#1}\fi");
  DefMacro!("\\b{}", r"\ifmmode\@math@baccent{#1}\else\@text@baccent{#1}\fi");

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

  DefPrimitive!("\\lx@thinmuskip", sub[stomach,(),state] {
    Tbox::new(arena::pin_static("\u{2009}"), None, None, Tokens!(T_CS!("\\,")),
      stored_map!("name"  => "thinspace", "isSpace" => true,
      "width" => state.lookup_value("\\thinmuskip")), state)
  });
  DefPrimitive!("\\lx@thinspace", sub[stomach,(),state] {
    Tbox::new(arena::pin_static("\u{2009}"), None, None, Tokens!(T_CS!("\\,")),
      stored_map!("name" => "thinspace", "width" => Dimension::from_str("0.16667em",state)?,
       "isSpace" => true), state)
  });
  DefMacro!(
    "\\,",
    r"\ifmmode\lx@thinmuskip\else\lx@thinspace\fi",
    protected => true
  );

  DefMacro!(
    "\\!",
    "\\ifmmode\\@math@negthinmuskip\\else\\@text@negthinmuskip\\fi"
  );

  DefPrimitive!("\\!", sub[stomach,(),state] {
    Tbox::new(arena::pin_static("\u{200B}"), None, None, Tokens!(T_CS!("\\!")),  // zero width space
      stored_map!("name"  => "negthinspace", "isSpace" => true,
      "width" => state.lookup_dimension("\\thinmuskip").unwrap().negate()), state)
  });

  DefPrimitive!("\\>", sub[stomach,(),state] {
    Tbox::new(arena::pin_static("\u{2005}"), None, None, Tokens!(T_CS!("\\>")),
      stored_map!("name"  => "medspace", "isSpace" => true,
      "width" => state.lookup_value("\\medmuskip")), state)
  });
  DefPrimitive!("\\;", sub[stomach, (), state] {
    Tbox::new(arena::pin_static("\u{2004}"), None, None, Tokens!(T_CS!("\\;")),
      stored_map!("name"  => "thickspace", "isSpace" => true,
      "width" => state.lookup_value("\\thickmuskip")), state)
  });

  Let!("\\:", "\\>");

  DefPrimitive!("\\ ", sub[stomach,(),state] {
    Tbox::new(arena::pin_static("\u{00A0}"), None, None, Tokens!(T_CS!("\\ ")),
      stored_map!("name" => "space", "isSpace" => true,
      "width" => Dimension::from_str("0.5em", state)?), state)
  });

  DefPrimitive!("\\\t", sub[stomach,(),state] {
    Tbox::new(arena::pin_static("\u{00A0}"), None, None, Tokens!(T_CS!("\\\t")),
      stored_map!("isSpace" => true, "width" => Dimension::from_str("1em",state)?), state)
  });

  DefPrimitive!("\\/", sub[stomach,(),state] {
    Tbox::new(arena::pin_static(""), None, None, Tokens!(T_CS!("\\/")),
      stored_map!("isSpace" => true, "name" => "italiccorr", "width" => Dimension::default()),state)
  });

});