use crate::package::*;
LoadDefinitions!({
//
// See amsfndoc
//
DefConstructor!("\\mathbb{}", "#1", bounded => true, require_math => true, scope => Some(Scope::Global),
  font => { family => "blackboard", series => "medium", shape => "upright" });
DefMacro!("\\Bbb{}",  "\\mathbb{#1}");
DefMacro!("\\bold{}", "\\mathbb{#1}");
// Also defined in eufrak
DefConstructor!("\\mathfrak{}", "#1", bounded => true, require_math => true, scope => Some(Scope::Global),
  font => { family => "fraktur", series => "medium", shape => "upright" });
DefMacro!("\\frak{}", "\\mathfrak{#1}");

// Not necessarily math
DefPrimitive!("\\checkmark",  "\u{2713}");    // CHECK MARK
DefPrimitive!("\\circledR",   "\u{00AE}");     // REGISTERED SIGN
DefPrimitive!("\\maltese",    "\u{2720}");    // MALTESE CROSs
DefPrimitive!("\\yen",        "\u{00A5}");     // YEN SIGN

// Math

// These are delimiters, but open or close??
DefMath!("\\ulcorner", "\u{231C}");    // TOP LEFT CORNER
DefMath!("\\urcorner", "\u{231D}");    // TOP RIGHT CORNER
DefMath!("\\llcorner", "\u{231E}");    // BOTTOM LEFT CORNER
DefMath!("\\lrcorner", "\u{231F}");    // BOTTOM RIGHT CORNER

DefMath!("\\dashrightarrow", "\u{21E2}", role => "ARROW");    // RIGHTWARDS DASHED ARROW
DefMath!("\\dashleftarrow",  "\u{21E0}", role => "ARROW");    // LEFTWARDS DASHED ARROW
DefMath!("\\dasharrow",      "\u{21E2}", role => "ARROW");    // RIGHTWARDS DASHED ARROW

DefMath!("\\square",  "\u{25A1}");                            // WHITE SQUARE
DefMath!("\\lozenge", "\u{25C6}");                            // WHITE DIAMOND

DefMath!("\\vartriangleright", "\u{22B3}");                   // CONTAINS AS NORMAL SUBGROUP (\rhd)
DefMath!("\\vartriangleleft",  "\u{22B2}");                   // NORMAL SUBGROUP OF (\lhd)

DefMath!("\\trianglerighteq", "\u{22B5}");    // CONTAINS AS NORMAL SUBGROUP OR EQUAL TO (\unrhd)
DefMath!("\\trianglelefteq",  "\u{22B4}");    // NORMAL SUBGROUP OF OR EQUAL TO (\unlhd)
DefMath!("\\rightsquigarrow", "\u{219D}", role => "ARROW");    // RIGHTWARDS WAVE ARROW

// amsfonts redefines various symbols already in TeX & LaTeX
// \widehat, \widetilde, \rightleftharpoons,\angle
// \hbar, \sqsubset, \sqsupset, \mho

// amsfonts also redefines these, unless latexsym is loaded.
// However, all these are already defined in TeX (from plain?)
// \Box, \Diamond, \leadsto, \lhd, \unlhd, \rhd, \unrhd

});