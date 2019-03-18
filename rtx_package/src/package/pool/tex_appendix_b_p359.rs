use crate::package::*;

LoadDefinitions!(state, {
  //======================================================================
  // TeX Book, Appendix B. p. 359

  // Ah, since \ldots can appear in text and math....

  DefConstructor!(
    "\\ldots",
    "?#isMath(<ltx:XMTok name='ldots' font='#font' role='ID'>\u{2026}</ltx:XMTok>)(\u{2026})"
  );
  // TODO
  // properties => properties!(sub[stomach, args, state] {
  //   if state.lookup_bool("IN_MATH") {
  //     font => state.lookup_font().merge(family => "serif", series => "medium", shape =>
  // "upright").specialize("\u{2026}")   }
  //  })
  // Since not DefMath!

  // And so can \vdots
  // DefConstructor('\vdots', undef,
  //   "?#isMath(<ltx:XMTok name='vdots' font='#font' role='ID'>\x{22EE}</ltx:XMTok>)(\x{22EE})",
  //   properties => sub {
  //     (LookupValue('IN_MATH')
  //       ? (font => LookupValue('font')->merge(family => 'serif',
  //           series => 'medium', shape => 'upright')->specialize("\x{22EE}"))
  //       : ()); });    # Since not DefMath!
  //                     # But not these!
  DefMathI!("\\cdots", None, "\u{22EF}", role => "ID"); // MIDLINE HORIZONTAL ELLIPSIS

  // DefMathI('\ddots', undef, "\x{22F1}", role => 'ID');           # DOWN RIGHT DIAGONAL ELLIPSIS
  // DefMathI('\colon', undef, ':',        role => 'METARELOP');    # Seems like good default role
  //         # Note that amsmath redefines \dots to be `smart'.
  //         # Aha, also can be in text...
  // DefConstructor('\dots', undef,
  //   "?#isMath(<ltx:XMTok name='dots' font='#font' role='ID'>\x{2026}</ltx:XMTok>)(\x{2026})",
  //   properties => sub {
  //     (LookupValue('IN_MATH')
  //       ? (font => LookupValue('font')->merge(family => 'serif',
  //           series => 'medium', shape => 'upright')->specialize("\x{2026}"))
  //       : ()); });    # Since not DefMath!

  // And while we're at it...

  // DefMathLigature("\u{22C5}\u{22C5}\u{22C5}" => "\u{22EF}", role => 'ID', name => 'cdots');

  DefLigature!(r"[.][.][.]", "\u{2026}", fontTest => sub[arg] {arg.get_family().unwrap_or(&Cow::Borrowed("")) != "typewriter" }); // ldots

  // TODO:
  // DefMathLigature("..." => "\x{2026}", role => 'ID', name => 'ldots');
  DefLigature!(r"--", "\u{2013}", // EN DASH (NOTE: With digits before &
    fontTest => sub[arg] { arg.get_family().unwrap_or(&Cow::Borrowed("")) != "typewriter" });
  // TODO
  //, aft => \N{FIGURE DASH})

  // EM DASH
  DefLigature!(r"---", "\u{2014}", fontTest => sub[arg] {arg.get_family().unwrap_or(&Cow::Borrowed("")) != "typewriter" });
  // Ligatures for doubled single left & right quotes to convert to double quotes
  // [should ligatures be part of a font, in the first place? (it is in TeX!)
  DefLigature!("\u{2018}\u{2018}", "\u{201C}", fontTest => sub[arg] {
    let family = arg.get_family().unwrap_or(&Cow::Borrowed(""));
    if family != "typewriter" {
      let encoding = arg.get_encoding().unwrap_or(&Cow::Borrowed("OT1"));
      encoding == "OT1" || encoding == "T1" } else {false} });
  DefLigature!("\u{2019}\u{2019}", "\u{201D}", fontTest => sub[arg] {
    let family = arg.get_family().unwrap_or(&Cow::Borrowed(""));
    if family != "typewriter" {
      let encoding = arg.get_encoding().unwrap_or(&Cow::Borrowed("OT1"));
      encoding == "OT1" || encoding == "T1" } else {false} });
});
