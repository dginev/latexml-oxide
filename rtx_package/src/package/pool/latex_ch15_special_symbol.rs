use crate::package::*;
LoadDefinitions!(state, {

//======================================================================
// C.15.3 Special Symbol
//======================================================================
DefMacro!("\\symbol{}", "\\char#1\\relax");

// These in LaTeX, but not in the book...
DefPrimitive!("\\textdollar", "$");
DefPrimitive!("\\textemdash", "\u{2014}");    // EM DASH
DefPrimitive!("\\textendash", "\u{2013}");    // EN DASH
DefPrimitive!("\\textexclamdown", "\u{00A1}");     // INVERTED EXCLAMATION MARK
DefPrimitive!("\\textquestiondown", "\u{00BF}");     // INVERTED QUESTION MARK
DefPrimitive!("\\textquotedblleft", "\u{201C}");    // LEFT DOUBLE QUOTATION MARK
DefPrimitive!("\\textquotedblright", "\u{201D}");    // RIGHT DOUBLE QUOTATION MARK
DefPrimitive!("\\textquotedbl", "\"");          // plain ascii DOUBLE QUOTATION
DefPrimitive!("\\textquoteleft", "\u{2018}");    // LEFT SINGLE QUOTATION MARK
DefPrimitive!("\\textquoteright", "\u{2019}");    // RIGHT SINGLE QUOTATION MARK
DefPrimitive!("\\textsterling", "\u{00A3}");     // POUND SIGN
DefPrimitive!("\\textasteriskcentered", "*");
DefPrimitive!("\\textbackslash", "\u{005C}");     // REVERSE SOLIDUS
DefPrimitive!("\\textbar", "|");
DefPrimitive!("\\textbraceleft", "{");
DefPrimitive!("\\textbraceright", "}");
DefPrimitive!("\\textbullet", "\u{2022}");    // BULLET
DefPrimitive!("\\textdaggerdbl", "\u{2021}");    // DOUBLE DAGGER
DefPrimitive!("\\textdagger", "\u{2020}");    // DAGGER
DefPrimitive!("\\textparagraph", "\u{00B6}");     // PILCROW SIGN
DefPrimitive!("\\textperiodcentered", "\u{22C5}");    // DOT OPERATOR
DefPrimitive!("\\textsection", "\u{00A7}");     // SECTION SIGN
DefAccent!("\\textcircled", "\u{0020DD}", "\u{0025EF}");          // Defined in TeX.pool
DefPrimitive!("\\textless", "<");
DefPrimitive!("\\textgreater", ">");
DefPrimitive!("\\textcopyright", "\u{00A9}");         // COPYRIGHT SIGN
DefPrimitive!("\\textasciicircum", "^");
DefPrimitive!("\\textasciitilde", "~");
DefPrimitive!("\\textcompwordmark", "");                // ???
DefPrimitive!("\\textunderscore", "_");
DefPrimitive!("\\textvisiblespace", "\u{2423}"); // SYMBOL FOR SPACE;  Not really the right symbol!
DefPrimitive!("\\textellipsis", "\u{2026}");   // HORIZONTAL ELLIPSIS
DefPrimitive!("\\textregistered", "\u{00AE}");    // REGISTERED SIGN
DefPrimitive!("\\texttrademark", "\u{2122}");   // TRADE MARK SIGN
DefConstructor!("\\textsuperscript{}", "<ltx:sup>#1</ltx:sup>",
  mode => "text".into_option());
// This is something coming from xetex/xelatex ? Why define this way?
//DefConstructor!("\\realsuperscript{}', "<ltx:text yoffset='0.5em' _noautoclose='1'>#1</ltx:text>");
DefConstructor!("\\realsuperscript{}", "<ltx:sup>#1</ltx:sup>",
  mode => "text".into_option());
DefPrimitive!("\\textordfeminine", "\u{00AA}");    // FEMININE ORDINAL INDICATOR
DefPrimitive!("\\textordmasculine", "\u{00BA}");    // MASCULINE ORDINAL INDICATOR
DefPrimitive!("\\SS", "SS");         // ?

DefMacro!("\\dag", "\\ifmmode{\\dagger}\\else\\textdagger\\fi");
DefMacro!("\\ddag", "\\ifmmode{\\ddagger}\\else\\textdaggerdbl\\fi");

DefConstructor!("\\sqrtsign Digested",
  "<ltx:XMApp><ltx:XMTok meaning='square-root'/><ltx:XMArg>#1</ltx:XMArg></ltx:XMApp>");

DefPrimitive!("\\mathparagraph", "\u{00B6}");
DefPrimitive!("\\mathsection", "\u{00A7}");
DefPrimitive!("\\mathdollar", '$');
DefPrimitive!("\\mathsterling", "\u{00A3}");
DefPrimitive!("\\mathunderscore", '_');
DefPrimitive!("\\mathellipsis", "\u{2026}");

// Are these glyph "pieces" or use alone?
// TODO
// DefMathI('\arrowvert', undef, "|",        role => 'VERTBAR');
// DefMathI('\Arrowvert', undef, "\u{2225}", role => 'VERTBAR');

// The following are glyph "pieces"...
DefPrimitive!("\\braceld", "\u{239D}");    //   left brace down part
DefPrimitive!("\\bracelu", "\u{239B}");    //   left brace up part
DefPrimitive!("\\bracerd", "\u{23A0}");    //   right brace down part
DefPrimitive!("\\braceru", "\u{239E}");    //   right brace up part

// DefMathI('\cdotp', undef, "\u{22C5}", role => 'MULOP');
// DefMathI('\ldotp', undef, ".",        role => 'MULOP');
// DefMathI('\intop', undef, "\u{222B}", role => 'INTOP', meaning => 'integral',
//   scriptpos => \&doScriptpos, mathstyle => \&doVariablesizeOp);
// DefMathI('\ointop', undef, "\u{222E}", role => 'INTOP', meaning => 'contour-integral',
//   scriptpos => \&doScriptpos, mathstyle => \&doVariablesizeOp);

// WHat are these? They look like superscripted parentheses, or combining accents!
// \lhook
// \rhook
Let!("\\gets", "\\leftarrow");

DefPrimitive!("\\lmoustache", "\u{23B0}");
DefPrimitive!("\\rmoustache", "\u{23B1}");
// TODO
// DefMathI('\mapstochar', undef, "\u{21A6}", role => 'ARROW', meaning => 'maps-to');
// DefMathI('\owns',       undef, "\u{220B}", role => 'RELOP', meaning => 'contains');

// \skew{}{}{} ????

// \symbol lookup symbol in font by index?


});
