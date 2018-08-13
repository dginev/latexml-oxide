use package::*;
pub fn load_definitions(core_state: &mut State) -> Result<()> {
  SetupBindingMacros!(core_state);

  //======================================================================
  // TeX Book, Appendix B. p. 359

  // Ah, since \ldots can appear in text and math....

  // DefConstructorI('\ldots', undef,
  //   "?#isMath(<ltx:XMTok name='ldots' font='#font' role='ID'>\x{2026}</ltx:XMTok>)(\x{2026})",
  //   properties => sub {
  //     (LookupValue('IN_MATH')
  //       ? (font => LookupValue('font')->merge(family => 'serif',
  //           series => 'medium', shape => 'upright')->specialize("\x{2026}"))
  //       : ()); });    # Since not DefMath!
  //                     # And so can \vdots
  // DefConstructorI('\vdots', undef,
  //   "?#isMath(<ltx:XMTok name='vdots' font='#font' role='ID'>\x{22EE}</ltx:XMTok>)(\x{22EE})",
  //   properties => sub {
  //     (LookupValue('IN_MATH')
  //       ? (font => LookupValue('font')->merge(family => 'serif',
  //           series => 'medium', shape => 'upright')->specialize("\x{22EE}"))
  //       : ()); });    # Since not DefMath!
  //                     # But not these!
  // DefMathI('\cdots', undef, "\x{22EF}", role => 'ID');    # MIDLINE HORIZONTAL ELLIPSIS

  // DefMathI('\ddots', undef, "\x{22F1}", role => 'ID');           # DOWN RIGHT DIAGONAL ELLIPSIS
  // DefMathI('\colon', undef, ':',        role => 'METARELOP');    # Seems like good default role
  //         # Note that amsmath redefines \dots to be `smart'.
  //         # Aha, also can be in text...
  // DefConstructorI('\dots', undef,
  //   "?#isMath(<ltx:XMTok name='dots' font='#font' role='ID'>\x{2026}</ltx:XMTok>)(\x{2026})",
  //   properties => sub {
  //     (LookupValue('IN_MATH')
  //       ? (font => LookupValue('font')->merge(family => 'serif',
  //           series => 'medium', shape => 'upright')->specialize("\x{2026}"))
  //       : ()); });    # Since not DefMath!

  // And while we're at it...

  // DefMathLigature("\u{22C5}\u{22C5}\u{22C5}" => "\u{22EF}", role => 'ID', name => 'cdots');

  DefLigature!("...", "\u{2026}"); //, fontTest => fontTest!(font, {font.get_family != "typewriter" }));  // ldots

  // DefMathLigature("..." => "\x{2026}", role => 'ID', name => 'ldots');

  //
  //**********************************************************************
  // Plain;  Extracted from Appendix B.
  //**********************************************************************
  //
  //======================================================================
  // TeX Book, Appendix B, p. 344
  //======================================================================
  // \dospecials ??
  //
  // Normally, the content branch contains the pure structure and meaning of a construct,
  // and the presentation is generated from lower level TeX macros that only concern
  // themselves with how to display the object.
  // Nevertheless, it is sometimes useful to know where the tokens in the presentation branch
  // came from;  particularly what their presumed "meaning" is.
  // For example, when search-indexing pmml, or providing links to definitions from the pmml.
  //
  // The following constructor (see how it's used in DefMath), adds meaning attributes
  // whereever it seems sensible on the presentation branch, after it has been generated.

  // DefConstructor('\@ASSERT@MEANING{}{}', '#2',
  //   reversion      => '#2',
  //   afterConstruct => sub {
  //     my ($document, $whatsit) = @_;
  //     my $node    = $document->getNode;              # This should be the wrapper just added.
  //     my $meaning = ToString($whatsit->getArg(1));
  //     addMeaningRec($document, $node, $meaning);
  //     $node; });

  //======================================================================
  // Properties for plain characters.
  // These are allowed in plain text, but need to act a bit special in math.
  DefMathI!('=', None, '=', role => v!("RELOP"),   meaning  => v!("equals"));
  DefMathI!('+', None, '+', role => v!("ADDOP"),   meaning  => v!("plus"));
  DefMathI!('-', None, '-', role => v!("ADDOP"),   meaning  => v!("minus"));
  DefMathI!('*', None, '*', role => v!("MULOP"),   meaning  => v!("times"));
  DefMathI!('/', None, '/', role => v!("MULOP"),   meaning  => v!("divide"));
  DefMathI!('!', None, '!', role => v!("POSTFIX"), meaning  => v!("factorial"));
  DefMathI!(',', None, ',', role => v!("PUNCT"));
  DefMathI!('.', None, '.', role => v!("PERIOD"));
  DefMathI!(';', None, ';', role => v!("PUNCT"));
  DefMathI!('(', None, '(', role => v!("OPEN"),    stretchy => false);
  DefMathI!(')', None, ')', role => v!("CLOSE"),   stretchy => false);
  DefMathI!('[', None, '[', role => v!("OPEN"),    stretchy => false);
  DefMathI!(']', None, ']', role => v!("CLOSE"),   stretchy => false);
  DefMathI!('|', None, '|', role => v!("VERTBAR"), stretchy => false);
  DefMathI!(':', None, ':', role => v!("METARELOP"), name => v!("colon")); // Seems like good default role
  DefMathI!('<', None, '<', role => v!("RELOP"), meaning => v!("less-than"));
  DefMathI!('>', None, '>', role => v!("RELOP"), meaning => v!("greater-than"));

  //======================================================================
  // TeX Book, Appendix B, p. 351

  // Old style font styles.
  // The trick is to create an empty Whatsit preserved till assimilation (for reversion'ing)
  // but to change the current font used in boxes.
  // (some of these were defined on different pages? or even latex...)
  Tag!("ltx:text", auto_open => true, auto_close => true);

  // Note that these, unlike \rmfamily, should set the other attributes to the defaults!
  DefPrimitiveI!("\\rm", noprimitive!(),
    font => Font!(family => "serif", series => "medium", shape => "upright"));
  DefPrimitiveI!("\\sf", noprimitive!(),
    font => Font!(family => "sansserif", series => "medium", shape => "upright"));
  DefPrimitiveI!("\\bf", noprimitive!(),
    font => Font!(series => "bold", family => "serif", shape => "upright"));
  DefPrimitiveI!("\\it", noprimitive!(),
    font => Font!(shape => "italic", family => "serif", series => "medium" ));
  DefPrimitiveI!("\\tt", noprimitive!(),
    font => Font!(family => "typewriter", series => "medium", shape => "upright" ));
  // No effect in math for the following 2 ?
  DefPrimitiveI!("\\sl", noprimitive!(),
    font => Font!(shape => "slanted", family => "serif", series => "medium" ));
  DefPrimitiveI!("\\sc", noprimitive!(),
    font => Font!(shape => "smallcaps", family => "serif", series => "medium" ));

  // Ideally, we should set these sizes from class files
  AssignValue!("NOMINAL_FONT_SIZE", 10);
  DefPrimitiveI!("\\tiny",         noprimitive!(), font => Font!(size => 5 ));
  DefPrimitiveI!("\\scriptsize",   noprimitive!(), font => Font!(size => 7 ));
  DefPrimitiveI!("\\footnotesize", noprimitive!(), font => Font!(size => 8 ));
  DefPrimitiveI!("\\small",        noprimitive!(), font => Font!(size => 9 ));
  DefPrimitiveI!("\\normalsize",   noprimitive!(), font => Font!(size => 10 ));
  DefPrimitiveI!("\\large",        noprimitive!(), font => Font!(size => 12 ));
  DefPrimitiveI!("\\Large",        noprimitive!(), font => Font!(size => 14.4 ));
  DefPrimitiveI!("\\LARGE",        noprimitive!(), font => Font!(size => 17.28 ));
  DefPrimitiveI!("\\huge",         noprimitive!(), font => Font!(size => 20.74 ));
  DefPrimitiveI!("\\Huge",         noprimitive!(), font => Font!(size => 29.8 ));

  DefPrimitiveI!("\\mit", noprimitive!(), require_math => true, font => Font!(family => "italic"));

  DefPrimitiveI!("\\frenchspacing", noprimitive!());
  DefPrimitiveI!("\\nonfrenchspacing", noprimitive!());
  // DefMacroI!("\\normalbaselines", undef,
  //   '\lineskip=\normallineskip\baselineskip=\normalbaselineskip\lineskiplimit=\normallineskiplimit');
  DefMacroI!(T_CS!("\\space"), None, T_SPACE!());
  DefMacroI!(T_CS!("\\lq"), None, T_OTHER!("`"));
  DefMacroI!(T_CS!("\\rq"), None, T_OTHER!("'"));
  Let!("\\empty", "\\@empty");
  //DefMacro!("\\null", "\hbox{}");
  Let!("\\bgroup", T_BEGIN!());
  Let!("\\egroup", T_END!());
  Let!("\\endgraf", "\\par");
  Let!("\\endline", "\\cr");

  DefPrimitiveI!("\\endline", noprimitive!());

  // Use \r for the newline from TeX!!!
  DefMacroI!(T_CS!("\\\r"), None, T_CS!("\\ ")); // \<cr> == \<space> Interesting (see latex.ltx)
  Let!(T_ACTIVE!("\r"), "\\par"); // (or is this just LaTeX?)

  Let!("\\\t", "\\\r"); // \<tab> == \<space>, also

  Ok(())
}
