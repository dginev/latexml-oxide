use crate::prelude::*;

LoadDefinitions!({
  //======================================================================
  // TeX Book, Appendix B. p. 359

  // Ah, since \ldots can appear in text and math....
  DefMacro!("\\ldots", "\\lx@ldots");
  DefConstructor!("\\lx@ldots",
    "?#isMath(<ltx:XMTok name='ldots' font='#font' role='ID'>\u{2026}</ltx:XMTok>)(\u{2026})",
    sizer      => "\u{2026}",
    reversion  => "\\ldots",
    properties => {
      if lookup_bool("IN_MATH") {
        let new_font = lookup_font().unwrap().merge(
          fontmap!(family => "serif", series => "medium", shape => "upright")
          .specialize("\u{2026}"));
        Ok(stored_map!("font" => new_font)) // Since not DefMath!
      } else {
        Ok(SymHashMap::default())
      }
  });

  DefConstructor!("\\vdots",
    "?#isMath(<ltx:XMTok name='vdots' font='#font' role='ID'>\u{22EE}</ltx:XMTok>)(\u{22EE})");
    // TODO:
    // properties => sub {
    //   (LookupValue('IN_MATH')
    //     ? (font => LookupValue('font')->merge(family => 'serif',
    //         series => 'medium', shape => 'upright')->specialize("\u{22EE}"))
    //     : ()); });    # Since not DefMath!
    //                   # But not these!
  DefMath!("\\cdots", None, "\u{22EF}", role => "ELIDEOP"); // MIDLINE HORIZONTAL ELLIPSIS
  DefMath!("\\ddots", None, "\u{22F1}", role => "ID");           // DOWN RIGHT DIAGONAL ELLIPSIS
  DefMath!("\\colon", None, ":",        role => "METARELOP");    // Seems like good default role
  //         # Note that amsmath redefines \dots to be `smart'.
  //         # Aha, also can be in text...
  DefConstructor!("\\dots",
    "?#isMath(<ltx:XMTok name='dots' font='#font' role='ID'>\u{2026}</ltx:XMTok>)(\u{2026})");
    // TODO:
  //   properties => sub {
  //     (LookupValue('IN_MATH')
  //       ? (font => LookupValue('font')->merge(family => 'serif',
  //           series => 'medium', shape => 'upright')->specialize("\u{2026}"))
  //       : ()); });    # Since not DefMath!

  // And while we're at it...

  // Pretest for XMath to keep from interpreting math that the DOM may not allow!!

  DefMathLigature!("\u{22C5}\u{22C5}\u{22C5}", "\u{22EF}", role => "ELIDEOP", name => "cdots");

  DefLigature!(r"[.][.][.]", "\u{2026}", fontTest => sub[arg] {arg.get_family().unwrap_or(&Cow::Borrowed("")) != "typewriter" }); // ldots

  DefMathLigature!("...", "\u{2026}", role => "ID", name => "ldots");

});
