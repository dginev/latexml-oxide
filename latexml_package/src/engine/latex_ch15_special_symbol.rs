use crate::prelude::*;

/// Perl: %unicode_enclosed_alphanumerics table
/// Maps single chars (0-9, a-z, A-Z) and numbers 10-20 to their circled Unicode equivalents.
fn unicode_enclosed_alphanumeric(text: &str) -> Option<String> {
  let ch = match text {
    "0" => '\u{24EA}', "1" => '\u{2460}', "2" => '\u{2461}', "3" => '\u{2462}',
    "4" => '\u{2463}', "5" => '\u{2464}', "6" => '\u{2465}', "7" => '\u{2466}',
    "8" => '\u{2467}', "9" => '\u{2468}', "10" => '\u{2469}', "11" => '\u{246A}',
    "12" => '\u{246B}', "13" => '\u{246C}', "14" => '\u{246D}', "15" => '\u{246E}',
    "16" => '\u{246F}', "17" => '\u{2470}', "18" => '\u{2471}', "19" => '\u{2472}',
    "20" => '\u{2473}',
    "a" => '\u{24D0}', "b" => '\u{24D1}', "c" => '\u{24D2}', "d" => '\u{24D3}',
    "e" => '\u{24D4}', "f" => '\u{24D5}', "g" => '\u{24D6}', "h" => '\u{24D7}',
    "i" => '\u{24D8}', "j" => '\u{24D9}', "k" => '\u{24DA}', "l" => '\u{24DB}',
    "m" => '\u{24DC}', "n" => '\u{24DD}', "o" => '\u{24DE}', "p" => '\u{24DF}',
    "q" => '\u{24E0}', "r" => '\u{24E1}', "s" => '\u{24E2}', "t" => '\u{24E3}',
    "u" => '\u{24E4}', "v" => '\u{24E5}', "w" => '\u{24E6}', "x" => '\u{24E7}',
    "y" => '\u{24E8}', "z" => '\u{24E9}',
    "A" => '\u{24B6}', "B" => '\u{24B7}', "C" => '\u{24B8}', "D" => '\u{24B9}',
    "E" => '\u{24BA}', "F" => '\u{24BB}', "G" => '\u{24BC}', "H" => '\u{24BD}',
    "I" => '\u{24BE}', "J" => '\u{24BF}', "K" => '\u{24C0}', "L" => '\u{24C1}',
    "M" => '\u{24C2}', "N" => '\u{24C3}', "O" => '\u{24C4}', "P" => '\u{24C5}',
    "Q" => '\u{24C6}', "R" => '\u{24C7}', "S" => '\u{24C8}', "T" => '\u{24C9}',
    "U" => '\u{24CA}', "V" => '\u{24CB}', "W" => '\u{24CC}', "X" => '\u{24CD}',
    "Y" => '\u{24CE}', "Z" => '\u{24CF}',
    _ => return None,
  };
  Some(ch.to_string())
}

LoadDefinitions!({
  //======================================================================
  // C.15.3 Special Symbol
  //======================================================================
  DefMacro!("\\symbol{}", "\\char#1\\relax");

  // These in LaTeX, but not in the book...
  DefPrimitive!("\\textdollar", "$");
  DefPrimitive!("\\textemdash", "\u{2014}"); // EM DASH
  DefPrimitive!("\\textendash", "\u{2013}"); // EN DASH
  DefPrimitive!("\\textexclamdown", "\u{00A1}"); // INVERTED EXCLAMATION MARK
  DefPrimitive!("\\textquestiondown", "\u{00BF}"); // INVERTED QUESTION MARK
  DefPrimitive!("\\textquotedblleft", "\u{201C}"); // LEFT DOUBLE QUOTATION MARK
  DefPrimitive!("\\textquotedblright", "\u{201D}"); // RIGHT DOUBLE QUOTATION MARK
  DefPrimitive!("\\textquotedbl", "\""); // plain ascii DOUBLE QUOTATION
  DefPrimitive!("\\textquoteleft", "\u{2018}"); // LEFT SINGLE QUOTATION MARK
  DefPrimitive!("\\textquoteright", "\u{2019}"); // RIGHT SINGLE QUOTATION MARK
  DefPrimitive!("\\textsterling", "\u{00A3}"); // POUND SIGN
  DefPrimitive!("\\textasteriskcentered", "*");
  DefPrimitive!("\\textbackslash", "\u{005C}"); // REVERSE SOLIDUS
  DefPrimitive!("\\textbar", "|");
  DefPrimitive!("\\textbraceleft", "{");
  DefPrimitive!("\\textbraceright", "}");
  DefPrimitive!("\\textbullet", "\u{2022}"); // BULLET
  DefPrimitive!("\\textdaggerdbl", "\u{2021}"); // DOUBLE DAGGER
  DefPrimitive!("\\textdagger", "\u{2020}"); // DAGGER
  DefPrimitive!("\\textparagraph", "\u{00B6}"); // PILCROW SIGN
  DefPrimitive!("\\textperiodcentered", "\u{00B7}"); // MIDDLE DOT
  DefPrimitive!("\\textsection", "\u{00A7}"); // SECTION SIGN
  // Perl: DefPrimitive('\textcircled {}', sub { ... })
  // Uses unicode_enclosed_alphanumerics table, falls back to combining circle U+20DD
  DefPrimitive!("\\textcircled {}", sub[(arg)] {
    let text = arg.to_string();
    let content = unicode_enclosed_alphanumeric(&text)
      .unwrap_or_else(|| format!("{}\u{20DD}", text));
    let in_math = lookup_bool("IN_MATH");
    let is_number = !text.is_empty() && text.chars().all(|c| c.is_ascii_digit());
    let mut props = stored_map!();
    if in_math {
      props.insert("role", Stored::from(if is_number { "NUMBER" } else { "UNKNOWN" }));
      props.insert("meaning", Stored::from(format!("circled-{}", text)));
    }
    Tbox::new(arena::pin(&content), None, None,
      Invocation!(T_CS!("\\textcircled"), vec![arg]),
      props)
  });
  // From latex_constructs.pool.ltxml
  DefAccent!("\\k", '\u{0328}', "\u{02DB}", below => true); // COMBINING OGONEK & OGONEK
  DefPrimitive!("\\textless", "<");
  DefPrimitive!("\\textgreater", ">");
  DefPrimitive!("\\textcopyright", "\u{00A9}"); // COPYRIGHT SIGN
  DefPrimitive!("\\textasciicircum", "^");
  DefPrimitive!("\\textasciitilde", "~");
  DefPrimitive!("\\textcompwordmark", ""); // ???
  DefPrimitive!("\\textunderscore", "_");
  // SYMBOL FOR SPACE;  Not really the right symbol!
  DefPrimitive!("\\textvisiblespace", "\u{2423}");
  DefPrimitive!("\\textellipsis", "\u{2026}"); // HORIZONTAL ELLIPSIS
  DefPrimitive!("\\textregistered", "\u{00AE}"); // REGISTERED SIGN
  DefPrimitive!("\\texttrademark", "\u{2122}"); // TRADE MARK SIGN
  DefConstructor!("\\textsuperscript{}", "<ltx:sup>#1</ltx:sup>",  mode => "text");
  DefConstructor!("\\textsubscript{}", "<ltx:sub>#1</ltx:sub>",  mode => "text");
  // This is something coming from xetex/xelatex ? Why define this way?
  //DefConstructor!("\\realsuperscript{}', "<ltx:text yoffset='0.5em'
  // _noautoclose='1'>#1</ltx:text>");
  DefConstructor!("\\realsuperscript{}", "<ltx:sup>#1</ltx:sup>",  mode => "text");
  DefPrimitive!("\\textordfeminine", "\u{00AA}"); // FEMININE ORDINAL INDICATOR
  DefPrimitive!("\\textordmasculine", "\u{00BA}"); // MASCULINE ORDINAL INDICATOR
  DefPrimitive!("\\SS", "SS"); // ?

  DefMacro!("\\dag", "\\ifmmode{\\dagger}\\else\\textdagger\\fi");
  DefMacro!("\\ddag", "\\ifmmode{\\ddagger}\\else\\textdaggerdbl\\fi");

  DefConstructor!(
    "\\sqrtsign Digested",
    "<ltx:XMApp><ltx:XMTok meaning='square-root'/><ltx:XMArg>#1</ltx:XMArg></ltx:XMApp>"
  );

  DefPrimitive!("\\mathparagraph", "\u{00B6}");
  DefPrimitive!("\\mathsection", "\u{00A7}");
  DefPrimitive!("\\mathdollar", "$");
  DefPrimitive!("\\mathsterling", "\u{00A3}");
  DefPrimitive!("\\mathunderscore", "_");
  DefPrimitive!("\\mathellipsis", "\u{2026}");

  // Perl: plain_constructs.pool.ltxml — glyph pieces that also work as delimiters
  DefMath!("\\arrowvert", None, "|", role => "VERTBAR");
  DefMath!("\\Arrowvert", None, "\u{2225}", role => "VERTBAR");

  // The following are glyph "pieces"...
  DefPrimitive!("\\braceld", "\u{239D}"); //   left brace down part
  DefPrimitive!("\\bracelu", "\u{239B}"); //   left brace up part
  DefPrimitive!("\\bracerd", "\u{23A0}"); //   right brace down part
  DefPrimitive!("\\braceru", "\u{239E}"); //   right brace up part

  // Perl: plain_constructs.pool.ltxml
  DefMath!("\\cdotp", None, "\u{22C5}", role => "MULOP");
  DefMath!("\\ldotp", None, ".", role => "MULOP");
  // Perl: latex_constructs.pool.ltxml — intop/ointop with dynamic scriptpos/mathstyle
  DefMath!("\\intop", None, "\u{222B}", role => "INTOP", meaning => "integral",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\ointop", None, "\u{222E}", role => "INTOP", meaning => "contour-integral",
    dynamic_scriptpos => true, dynamic_mathstyle => true);

  // WHat are these? They look like superscripted parentheses, or combining accents!
  // \lhook
  // \rhook
  Let!("\\gets", "\\leftarrow");

  DefPrimitive!("\\lmoustache", "\u{23B0}");
  DefPrimitive!("\\rmoustache", "\u{23B1}");
  // Perl: plain_constructs.pool.ltxml
  DefMath!("\\mapstochar", None, "\u{21A6}", role => "ARROW", meaning => "maps-to");
  DefMath!("\\owns", None, "\u{220B}", role => "RELOP", meaning => "contains");

  // \symbol lookup symbol in font by index?
});
