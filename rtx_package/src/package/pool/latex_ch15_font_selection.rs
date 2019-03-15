use crate::package::*;

LoadDefinitions!(outer_state, {
  //**********************************************************************
  // C.15 Font Selection
  //**********************************************************************
  //======================================================================
  // C.15.1 Changing the Type Style
  //======================================================================
  // Text styles.

  DefMacro!("\\rmdefault", "cmr");
  DefMacro!("\\sfdefault", "cmss");
  DefMacro!("\\ttdefault", "cmtt");
  DefMacro!("\\bfdefault", "bx");
  DefMacro!("\\mddefault", "m");
  DefMacro!("\\itdefault", "it");
  DefMacro!("\\sldefault", "sl");
  DefMacro!("\\scdefault", "sc");
  DefMacro!("\\updefault", "n");
  DefMacro!("\\encodingdefault", "OT1");
  DefMacro!("\\familydefault", "\\rmdefault");
  DefMacro!("\\seriesdefault", "\\mddefault");
  DefMacro!("\\shapedefault", "\\updefault");

  Let!("\\mediumseries", "\\mdseries");
  Let!("\\normalshape", "\\upshape");

  // ? DefMacro("\\f@encoding','cm');
  DefMacro!("\\f@family", "cm");
  DefMacro!("\\f@series", "");
  DefMacro!("\\f@shape", "");
  DefMacro!("\\f@size", "");

  // These do NOT immediately effect the font!
  DefMacro!("\\fontfamily{}", "\\edef\\f@family{#1}");
  DefMacro!("\\fontseries{}", "\\edef\\f@series{#1}");
  DefMacro!("\\fontshape{}", "\\edef\\f@shape{#1}");

  // For fonts not allowed in math!!!
  DefPrimitive!("\\not@math@alphabet@@ {}", sub[stomach, args, inner_state] {
    if inner_state.lookup_bool("IN_MATH") {
      unpack_to_string!(args => c);
      let message = s!("Command {:?} invalid in math mode", c);
      Warn!("unexpected", c, stomach, inner_state, message);
    }
    Ok(vec![])
  });

  // These DO immediately effect the font!
  DefMacro!("\\mdseries", "\\not@math@alphabet@@{\\mddefault}\\fontseries{\\mddefault}\\selectfont");
  DefMacro!("\\bfseries", "\\not@math@alphabet@@{\\bfdefault}\\fontseries{\\bfdefault}\\selectfont");

  DefMacro!("\\rmfamily", "\\not@math@alphabet@@{\\rmdefault}\\fontfamily{\\rmdefault}\\selectfont");
  DefMacro!("\\sffamily", "\\not@math@alphabet@@{\\sfdefault}\\fontfamily{\\sfdefault}\\selectfont");
  DefMacro!("\\ttfamily", "\\not@math@alphabet@@{\\ttdefault}\\fontfamily{\\ttdefault}\\selectfont");

  DefMacro!("\\upshape", "\\not@math@alphabet@@{\\updefault}\\fontshape{\\updefault}\\selectfont");
  DefMacro!("\\itshape", "\\not@math@alphabet@@{\\itdefault}\\fontshape{\\itdefault}\\selectfont");
  DefMacro!("\\slshape", "\\not@math@alphabet@@{\\sldefault}\\fontshape{\\sldefault}\\selectfont");
  DefMacro!("\\scshape", "\\not@math@alphabet@@{\\scdefault}\\fontshape{\\scdefault}\\selectfont");

  DefMacro!(
    "\\normalfont",
    "\\fontfamily{\\rmdefault}\\fontseries{\\mddefault}\\fontshape{\\updefault}\\selectfont"
  );
  DefMacro!(
    "\\verbatim@font",
    "\\fontfamily{\\ttdefault}\\fontseries{\\mddefault}\\fontshape{\\updefault}\\selectfont"
  );

  Let!("\\reset@font", "\\normalfont");

  DefPrimitive!("\\selectfont", sub[stomach, args, inner_state] {
    let mut gullet = stomach.get_gullet_mut();
    let family = Expand!(T_CS!("\\f@family"), gullet).to_string();
    let series = Expand!(T_CS!("\\f@series"),gullet).to_string();
    let shape  = Expand!(T_CS!("\\f@shape"), gullet).to_string();
    if let Some(sh) = font::lookup_font_family(&family) { MergeFont!(sh.clone()); }
    else {
      let message = s!("Unrecognized font family {:?}.", family);
      Info!("unexpected", family, stomach, inner_state, message); }
    if let Some(sh) = font::lookup_font_series(&series) { MergeFont!(sh.clone()); }
    else {
      let message = s!("Unrecognized font series {:?}.", series);
      Info!("unexpected", series, stomach, inner_state, message); }
    if let Some(sh) = font::lookup_font_shape(&shape) { MergeFont!(sh.clone()); }
    else {
      let message = s!("Unrecognized font shape {:?}.", shape);
      Info!("unexpected",shape, stomach, inner_state, message); }
    Ok(vec![])
  });

  DefMacro!(
    "\\usefont{}{}{}{}",
    "\\fontencoding{#1}\\fontfamily{#2}\\fontseries{#3}\\fontshape{#4}\\selectfont"
  );

  // // TODO:
  // // If these series or shapes appear in math, they revert it to roman, medium, upright (?)
  // DefConstructor!("\\textmd@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
  //   bounded      => 1, font => { series => "medium" }, alias => "\\textmd",
  //   beforeDigest => sub { DefMacro("\\f@series", "m"); });
  // DefConstructor!("\\textbf@math{}", "<ltx:text _noautoclose="1">#1</ltx:text>", mode => "text",
  //   bounded      => 1, font => { series => "bold" }, alias => "\\textbf",
  //   beforeDigest => sub { DefMacro("\\f@series", "b"); });
  // DefConstructor!("\\textrm@math{}", "<ltx:text _noautoclose="1">#1</ltx:text>", mode => "text",
  //   bounded      => 1, font => { family => "serif" }, alias => "\\textrm",
  //   beforeDigest => sub { DefMacro("\\f@family", "cm"); });
  // DefConstructor!("\\textsf@math{}", "<ltx:text _noautoclose="1">#1</ltx:text>", mode => "text",
  //   bounded      => 1, font => { family => "sansserif" }, alias => "\\textsf",
  //   beforeDigest => sub { DefMacro("\\f@family", "cmss"); });
  // DefConstructor!("\\texttt@math{}", "<ltx:text _noautoclose="1">#1</ltx:text>", mode => "text",
  //   bounded      => 1, font => { family => "typewriter" }, alias => "\\texttt",
  //   beforeDigest => sub { DefMacro("\\f@family", "cmtt"); });

  // DefConstructor!("\\textup@math{}", "<ltx:text _noautoclose="1">#1</ltx:text>", mode => "text",
  //   bounded      => 1, font => { shape => "upright" }, alias => "\\textup",
  //   beforeDigest => sub { DefMacro("\\f@shape", ""); });
  // DefConstructor!("\\textit@math{}", "<ltx:text _noautoclose="1">#1</ltx:text>", mode => "text",
  //   bounded      => 1, font => { shape => "italic" }, alias => "\\textit",
  //   beforeDigest => sub { DefMacro("\\f@shape", "i"); });
  // DefConstructor!("\\textsl@math{}", "<ltx:text _noautoclose="1">#1</ltx:text>", mode => "text",
  //   bounded      => 1, font => { shape => "slanted" }, alias => "\\textsl",
  //   beforeDigest => sub { DefMacro("\\f@shape", "sl"); });
  // DefConstructor!("\\textsc@math{}", "<ltx:text _noautoclose="1">#1</ltx:text>", mode => "text",
  //   bounded      => 1, font => { shape => "smallcaps" }, alias => "\\textsc",
  //   beforeDigest => sub { DefMacro("\\f@shape", "sc"); });
  // DefConstructor!("\\textnormal@math{}", "<ltx:text _noautoclose="1">#1</ltx:text>", mode =>
  // "text",   bounded => 1, font => { family => "serif", series => "medium", shape => "upright"
  // }, alias => "\\textnormal",   beforeDigest => sub { DefMacro("\\f@family", "cmtt");
  //     DefMacro("\\f@series", "m");
  //     DefMacro("\\f@shape",  "n"); });

  // These really should be robust! which is a source of expand timing issues!
  DefMacro!("\\textmd{}",     "\\ifmmode\\textmd@math{#1}\\else{\\mdseries #1}\\fi",       protected => true);
  DefMacro!("\\textbf{}",     "\\ifmmode\\textbf@math{#1}\\else{\\bfseries #1}\\fi",       protected => true);
  DefMacro!("\\textrm{}",     "\\ifmmode\\textrm@math{#1}\\else{\\rmfamily #1}\\fi",       protected => true);
  DefMacro!("\\textsf{}",     "\\ifmmode\\textsf@math{#1}\\else{\\sffamily #1}\\fi",       protected => true);
  DefMacro!("\\texttt{}",     "\\ifmmode\\texttt@math{#1}\\else{\\ttfamily #1}\\fi",       protected => true);
  DefMacro!("\\textup{}",     "\\ifmmode\\textup@math{#1}\\else{\\upshape #1}\\fi",        protected => true);
  DefMacro!("\\textit{}",     "\\ifmmode\\textit@math{#1}\\else{\\itshape #1}\\fi",        protected => true);
  DefMacro!("\\textsl{}",     "\\ifmmode\\textsl@math{#1}\\else{\\slshape #1}\\fi",        protected => true);
  DefMacro!("\\textsc{}",     "\\ifmmode\\textsc@math{#1}\\else{\\scshape #1}\\fi",        protected => true);
  DefMacro!("\\textnormal{}", "\\ifmmode\\textnormal@math{#1}\\else{\\normalfont #1}\\fi", protected => true);
});
