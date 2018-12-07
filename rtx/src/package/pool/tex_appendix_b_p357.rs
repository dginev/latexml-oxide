use crate::package::*;

pub fn load_definitions(core_state: &mut State) -> Result<()> {
  SetupBindingMacros!(core_state);

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

  Ok(())
}
