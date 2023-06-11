use crate::package::*;

LoadDefinitions!(state, {
  //----------------------------------------------------------------------
  // Math Accents.
  //----------------------------------------------------------------------
  // LaTeX; Table 3.11. Math Mode Accents, p.50.
  // Are these all TeX (or LaTeX)?
  // Note that most of these should NOT be stretchy, by default!
  DefMath!("\\hat Digested", "\u{005E}",
    operator_role => "OVERACCENT", operator_stretchy => false);
  DefMath!("\\check Digested", "\u{02C7}",
    operator_role => "OVERACCENT", operator_stretchy => false);    // CARON
  DefMath!("\\breve Digested", "\u{02D8}", operator_role => "OVERACCENT");    // BREVE
  DefMath!("\\acute Digested", "\u{00B4}",  operator_role => "OVERACCENT");    // ACUTE ACCENT
  DefMath!("\\grave Digested", "\u{0060}",  operator_role => "OVERACCENT");    // GRAVE ACCENT
  DefMath!("\\tilde Digested", "\u{007E}",
    operator_role => "OVERACCENT", operator_stretchy => false);           // TILDE
  DefMath!("\\bar Digested", "\u{00AF}",
    operator_role => "OVERACCENT", operator_stretchy => false);           // MACRON
  DefMath!("\\vec Digested", "\u{2192}",
    operator_role => "OVERACCENT", operator_stretchy => false);           // RIGHTWARDS ARROW
  DefMath!("\\dot Digested",      "\u{02D9}", operator_role => "OVERACCENT");    // DOT ABOVE
  DefMath!("\\ddot Digested",     "\u{00A8}",  operator_role => "OVERACCENT");    // DIAERESIS
  DefMath!("\\overline Digested", "\u{00AF}",  operator_role => "OVERACCENT");    // MACRON
  DefMath!("\\widehat Digested", "\u{005E}", operator_role => "OVERACCENT"); // CIRCUMFLEX ACCENT [plain? also amsfonts]
  DefMath!("\\widetilde Digested", "\u{007E}", operator_role => "OVERACCENT"); // TILDE [plain? also amsfonts]
  // These aren"t handled as simple accents by TeX, so no Digested
  DefMath!("\\overbrace {}", "\u{23DE}", operator_role => "OVERACCENT",       // TOP CURLY BRACKET
    scriptpos => "mid", robust => true);
  DefMath!("\\underbrace {}", "\u{23DF}", operator_role => "UNDERACCENT",     // BOTTOM CURLY BRACKET
    scriptpos => "mid", robust => true);

  // NOTE that all the above accents REQUIRE math mode
  // EXCEPT underline, overrightarrow and overleftarrow!

  DefMath!("\\math@underline{}", "\u{00AF}", operator_role => "UNDERACCENT",
    name => "underline", alias => "\\underline");
  DefConstructor!("\\text@underline{}", "<ltx:text framed='underline' _noautoclose='true'>#1</ltx:text>");
  DefMath!("\\math@overrightarrow{}", "\u{2192}", operator_role => "OVERACCENT",
    name => "overrightarrow", alias => "\\overrightarrow");
  DefMath!("\\math@overleftarrow{}", "\u{2190}", operator_role => "OVERACCENT",
    name => "overleftarrow", alias => "\\overleftarrow");

  // Careful: Use \protect so that it doesn"t expand too early in alignments, etc.
  DefMacro!("\\underline{}", r"\protect\ifmmode\math@underline{#1}\else\text@underline{#1}\fi");
  Let!("\\underbar", "\\underline");    // Will anyone notice?

  DefMacro!("\\overrightarrow{}", r"\protect\ifmmode\math@overrightarrow{#1}\else$\math@overrightarrow{#1}$\fi");
  DefMacro!("\\overleftarrow{}", r"\protect\ifmmode\math@overleftarrow{#1}\else$\math@overleftarrow{#1}$\fi");

  DefMacro!("\\skew{}{}{}", r"{#2{#3\mkern#1mu}\mkern-#1mu}{}");  // ignore the subtle spacing for now?

});
