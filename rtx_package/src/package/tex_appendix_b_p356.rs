use crate::package::*;

LoadDefinitions!(state, {
    //======================================================================
  // TeX Book, Appendix B. p. 356

  DefPrimitive!("\\raggedright",   None);
  DefPrimitive!("\\raggedleft",    None);    // this is actually LaTeX
  DefPrimitive!("\\ttraggedright", None);
  DefPrimitive!("\\leavevmode",    None);
  DefMacro!("\\mathhexbox{}{}{}", r##"\leavevmode\hbox{$\m@th \mathchar"#1#2#3$}"##);

  //----------------------------------------------------------------------
  // Actually from LaTeX; Table 3.2. Non-English Symbols, p.39

  // The following shouldn't appear in math.
  DefPrimitive!("\\OE", "\u{0152}");    // LATIN CAPITAL LIGATURE OE
  DefPrimitive!("\\oe", "\u{0153}");    // LATIN SMALL LIGATURE OE
  DefPrimitive!("\\AE", "\u{00C6}");     // LATIN CAPITAL LETTER AE
  DefPrimitive!("\\ae", "\u{00E6}");     // LATIN SMALL LETTER AE
  DefPrimitive!("\\AA", "\u{00C5}");     // LATIN CAPITAL LETTER A WITH RING ABOVE
  DefPrimitive!("\\aa", "\u{00E5}");     // LATIN SMALL LETTER A WITH RING ABOVE
  DefPrimitive!("\\O",  "\u{00D8}");     // LATIN CAPITAL LETTER O WITH STROKE
  DefPrimitive!("\\o",  "\u{00F8}");     // LATIN SMALL LETTER O WITH STROKE
  DefPrimitive!("\\L",  "\u{0141}");    // LATIN CAPITAL LETTER L WITH STROKE
  DefPrimitive!("\\l",  "\u{0142}");    // LATIN SMALL LETTER L WITH STROKE
  DefPrimitive!("\\ss", "\u{00DF}");     // LATIN SMALL LETTER SHARP S

  // apparently the rest can appear in math.
  DefPrimitive!("\\lx@sectionsign",   "\u{00a7}");    // SECTION SIGN
  DefPrimitive!("\\lx@paragraphsign", "\u{00B6}");    // PILCROW SIGN
  DefMacro!("\\S", "\\lx@sectionsign");
  DefMacro!("\\P", "\\lx@paragraphsign");
  DefPrimitive!("\\dag",       "\u{2020}");          // DAGGER
  DefPrimitive!("\\ddag",      "\u{2021}");          // DOUBLE DAGGER
  DefPrimitive!("\\copyright", "\u{00A9}");           // COPYRIGHT SIGN
  DefPrimitive!("\\pounds",    "\u{00A3}");           // POUND SIGN

});
