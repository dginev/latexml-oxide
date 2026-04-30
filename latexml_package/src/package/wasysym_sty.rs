use crate::prelude::*;

LoadDefinitions!({
  // Math symbols
  DefMath!("\\Join", "\u{2A1D}");
  DefMath!("\\Box", "\u{25A1}");
  DefMath!("\\Diamond", "\u{25C7}");
  DefMath!("\\leadsto", "\u{2933}");
  DefMath!("\\sqsubset", "\u{228F}");
  DefMath!("\\sqsupset", "\u{2290}");
  DefMath!("\\lhd", "\u{22B2}");
  DefMath!("\\unlhd", "\u{22B4}");
  DefMath!("\\LHD", "\u{25C0}");
  DefMath!("\\rhd", "\u{22B3}");
  DefMath!("\\unrhd", "\u{22B5}");
  DefMath!("\\RHD", "\u{25B6}");
  DefMath!("\\apprle", "\u{2272}");
  DefMath!("\\apprge", "\u{2273}");
  DefMath!("\\wasypropto", "\u{221D}");
  DefMath!("\\invneg", "\u{2310}");
  DefMath!("\\ocircle", "\u{25CB}");
  DefMath!("\\logof", "\u{229B}");

  // Feynman diagrams
  DefPrimitive!("\\photon", "\u{3030}\u{3030}");
  DefPrimitive!("\\gluon", "\u{27BF}\u{27BF}\u{27BF}");

  // Integral variants
  Let!("\\varint", "\\int");
  Let!("\\varoint", "\\oint");
  DefMath!("\\iintop", "\u{222C}", meaning => "double-integral",
    role => "INTOP", dynamic_mathstyle => true);
  DefMath!("\\iiintop", "\u{222D}", meaning => "triple-integral",
    role => "INTOP", dynamic_mathstyle => true);
  DefMath!("\\oiintop", "\u{222F}", meaning => "surface-integral",
    role => "INTOP", dynamic_mathstyle => true);
  DefMath!("\\iint", "\u{222C}", meaning => "double-integral",
    role => "INTOP", dynamic_mathstyle => true);
  DefMath!("\\iiint", "\u{222D}", meaning => "triple-integral",
    role => "INTOP", dynamic_mathstyle => true);
  DefMath!("\\oiint", "\u{222F}", meaning => "surface-integral",
    role => "INTOP", dynamic_mathstyle => true);

  // Gender / miscellaneous symbols
  DefPrimitive!("\\male", "\u{2642}");
  DefPrimitive!("\\female", "\u{2640}");
  DefPrimitive!("\\currency", "\u{00A4}");
  DefPrimitive!("\\phone", "\u{260E}");
  DefPrimitive!("\\recorder", "\u{2315}");
  DefPrimitive!("\\clock", "\u{1F552}");
  DefPrimitive!("\\lightning", "\u{21AF}");
  DefPrimitive!("\\pointer", "\u{21E8}");
  DefPrimitive!("\\RIGHTarrow", "\u{25B6}");
  DefPrimitive!("\\LEFTarrow", "\u{25C0}");
  DefPrimitive!("\\UParrow", "\u{25B2}");
  DefPrimitive!("\\DOWNarrow", "\u{25BC}");
  DefPrimitive!("\\diameter", "\u{2300}");
  DefPrimitive!("\\invdiameter", "\u{29B0}");
  DefPrimitive!("\\varangle", "\u{2222}");
  DefPrimitive!("\\wasylozenge", "\u{2311}");
  DefPrimitive!("\\kreuz", "\u{2720}");
  DefPrimitive!("\\smiley", "\u{263A}");
  DefPrimitive!("\\frownie", "\u{2639}");
  DefPrimitive!("\\blacksmiley", "\u{263B}");
  DefPrimitive!("\\sun", "\u{263C}");
  DefPrimitive!("\\checked", "\u{2713}");
  DefPrimitive!("\\bell", "\u{237E}");
  // ataribox — special: colored text (White on Black background)
  // Perl: font => { color => White, background => Black }, bounded => 1
  DefPrimitive!("\\ataribox", "\u{26CB}",
  bounded => true, font => sub[_f] {
    use latexml_core::common::color;
    Ok(Font {
      color: Some(color::WHITE),
      bg: Some(color::BLACK),
      ..Font::default()
    })
  });
  DefPrimitive!("\\cent", "\u{00A2}");
  DefPrimitive!("\\permil", "\u{2030}");
  DefPrimitive!("\\brokenvert", "\u{00A6}");
  DefPrimitive!("\\wasytherefore", "\u{2234}");
  DefPrimitive!("\\Bowtie", "\u{22C8}");
  DefPrimitive!("\\agemO", "\u{2127}");

  // Electrical symbols
  DefPrimitive!("\\AC", "\u{223C}");
  DefPrimitive!("\\HF", "\u{2248}");
  DefPrimitive!("\\VHF", "\u{224B}");

  // Geometric shapes
  DefPrimitive!("\\Square", "\u{25A1}");
  DefPrimitive!("\\XBox", "\u{2327}");
  DefPrimitive!("\\CheckedBox", "\u{2611}");
  DefPrimitive!("\\hexagon", "\u{2394}");
  DefPrimitive!("\\varhexagon", "\u{2B21}");
  DefMacro!("\\octagon", "\\lx@nounicode{\\octagon}");
  DefPrimitive!("\\pentagon", "\u{2B20}");
  DefPrimitive!("\\hexstar", "\u{26B9}");
  DefPrimitive!("\\varhexstar", "\u{26B9}");
  DefPrimitive!("\\davidsstar", "\u{2721}");

  // Musical notes
  DefPrimitive!("\\eighthnote", "\u{1D160}");
  DefPrimitive!("\\quarternote", "\u{1D15F}");
  DefPrimitive!("\\halfnote", "\u{1D15E}");
  DefPrimitive!("\\fullnote", "\u{1D15D}");
  DefPrimitive!("\\twonotes", "\u{266B}");

  // Circles
  DefPrimitive!("\\Circle", "\u{25CB}");
  DefPrimitive!("\\CIRCLE", "\u{25CF}");
  DefMacro!("\\Leftcircle", "\\lx@nounicode{\\Leftcircle}");
  DefMacro!("\\Rightcircle", "\\lx@nounicode{\\Rightcircle}");
  DefPrimitive!("\\LEFTCIRCLE", "\u{25D6}");
  DefPrimitive!("\\RIGHTCIRCLE", "\u{25D7}");
  DefPrimitive!("\\LEFTcircle", "\u{25D0}");
  DefPrimitive!("\\RIGHTcircle", "\u{25D1}");
  DefPrimitive!("\\leftturn", "\u{21BA}");
  DefPrimitive!("\\rightturn", "\u{21BB}");

  // Phonetic / special chars
  DefPrimitive!("\\thorn", "\u{00FE}");
  DefPrimitive!("\\Thorn", "\u{00DE}");
  DefPrimitive!("\\openo", "\u{0254}");
  DefPrimitive!("\\inve", "\u{01DD}");

  // Astronomical symbols
  DefPrimitive!("\\vernal", "\u{2648}");
  DefPrimitive!("\\ascnode", "\u{260A}");
  DefPrimitive!("\\descnode", "\u{260B}");
  DefPrimitive!("\\fullmoon", "\u{1F315}");
  DefPrimitive!("\\newmoon", "\u{1F311}");
  DefPrimitive!("\\leftmoon", "\u{263E}");
  DefPrimitive!("\\rightmoon", "\u{263D}");
  DefPrimitive!("\\astrosun", "\u{2609}");
  DefPrimitive!("\\mercury", "\u{263F}");
  DefPrimitive!("\\venus", "\u{2640}");
  DefPrimitive!("\\earth", "\u{2641}");
  DefPrimitive!("\\mars", "\u{2642}");
  DefPrimitive!("\\jupiter", "\u{2643}");
  DefPrimitive!("\\saturn", "\u{2644}");
  DefPrimitive!("\\uranus", "\u{26E2}");
  DefPrimitive!("\\neptune", "\u{2646}");
  DefPrimitive!("\\pluto", "\u{2647}");

  // Zodiac symbols
  DefPrimitive!("\\aries", "\u{2648}");
  DefPrimitive!("\\taurus", "\u{2649}");
  DefPrimitive!("\\gemini", "\u{264A}");
  DefPrimitive!("\\cancer", "\u{264B}");
  DefPrimitive!("\\leo", "\u{264C}");
  DefPrimitive!("\\virgo", "\u{264D}");
  DefPrimitive!("\\libra", "\u{264E}");
  DefPrimitive!("\\scorpio", "\u{264F}");
  DefPrimitive!("\\sagittarius", "\u{2650}");
  DefPrimitive!("\\capricornus", "\u{2651}");
  DefPrimitive!("\\aquarius", "\u{2652}");
  DefPrimitive!("\\pisces", "\u{2653}");
  DefPrimitive!("\\conjunction", "\u{260C}");
  DefPrimitive!("\\opposition", "\u{260D}");

  // APL symbols
  DefPrimitive!("\\APLcomment", "\u{235D}");
  DefPrimitive!("\\APLstar", "\u{22C6}");
  DefPrimitive!("\\APLlog", "\u{235F}");
  DefPrimitive!("\\APLbox", "\u{2395}");
  DefPrimitive!("\\APLup", "\u{234B}");
  DefPrimitive!("\\APLdown", "\u{2352}");
  DefPrimitive!("\\APLinput", "\u{235E}");
  DefPrimitive!("\\APLinv", "\u{2339}");
  DefPrimitive!("\\APLuparrowbox", "\u{2350}");
  DefPrimitive!("\\APLdownarrowbox", "\u{2357}");
  DefPrimitive!("\\APLleftarrowbox", "\u{2347}");
  DefPrimitive!("\\APLrightarrowbox", "\u{2348}");
  DefPrimitive!("\\notbackslash", "\u{2340}");
  DefPrimitive!("\\notslash", "\u{233F}");

  // APL modifiers
  DefPrimitive!("\\text@tilde", "\u{007E}");
  DefPrimitive!("\\text@circ", "\u{2218}");
  DefMacro!(
    "\\APLnot{}",
    "\\lx@kludged{#1\\lx@tweaked{width=0pt,xoffset=-0.8em}{\\text@tilde}}"
  );
  DefMacro!(
    "\\APLvert{}",
    "\\lx@kludged{#1\\lx@tweaked{width=0pt,xoffset=-1em}{|}}"
  );
  DefMacro!(
    "\\APLcirc{}",
    "\\lx@kludged{#1\\lx@tweaked{width=0pt,xoffset=-0.66em}{\\text@circ}}"
  );
  DefMacro!("\\APLminus", "\\raise 0.5ex \\hbox{-}");
});
