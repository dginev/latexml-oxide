use crate::prelude::*;
LoadDefinitions!({
  //======================================================================
  // Accent Symbols
  DefAccent!("\\capitalacute", '\u{0301}', "\u{00B4}");
  DefAccent!("\\capitalbreve", '\u{0306}', "\u{02D8}");
  DefAccent!("\\capitalcaron", '\u{030C}', "\u{02C7}");
  DefAccent!("\\capitalcedilla",      '\u{0327}', "\u{00B8}", below => true);
  DefAccent!("\\capitalcircumflex", '\u{0302}', "\u{02C6}");
  DefAccent!("\\capitaldieresis", '\u{0308}', "\u{00A8}");
  DefAccent!("\\capitaldotaccent", '\u{0307}', "\u{02D9}");
  DefAccent!("\\capitalgrave", '\u{0300}', "\u{0060}");
  DefAccent!("\\capitalhungarumlaut", '\u{030B}', "\u{02DD}");
  DefAccent!("\\capitalmacron", '\u{0304}', "\u{00AF}");
  DefAccent!("\\capitalnewtie", '\u{0361}', "-");
  DefAccent!("\\capitalogonek", '\u{0328}', "\u{02DB}");
  DefAccent!("\\capitalring", '\u{030A}', "\u{02DA}");
  DefAccent!("\\capitaltie", '\u{0361}', "-");
  DefAccent!("\\capitaltilde", '\u{0303}', "\u{02DC}");
  DefAccent!("\\newtie", '\u{0361}', "-");

  //======================================================================
  // Numerals
  DefPrimitive!("\\textonesuperior",   "\u{00B9}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\texttwosuperior",   "\u{00B2}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textthreesuperior", "\u{00B3}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textonequarter",    "\u{00BC}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textonehalf",       "\u{00BD}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textthreequarters", "\u{00BE}",
    bounded => true, font => { encoding => "TS1" });

  // Old-style numerals
  DefPrimitive!("\\textzerooldstyle",  "0", font => { family => "oldstyle" });
  DefPrimitive!("\\textoneoldstyle",   "1", font => { family => "oldstyle" });
  DefPrimitive!("\\texttwooldstyle",   "2", font => { family => "oldstyle" });
  DefPrimitive!("\\textthreeoldstyle", "3", font => { family => "oldstyle" });
  DefPrimitive!("\\textfouroldstyle",  "4", font => { family => "oldstyle" });
  DefPrimitive!("\\textfiveoldstyle",  "5", font => { family => "oldstyle" });
  DefPrimitive!("\\textsixoldstyle",   "6", font => { family => "oldstyle" });
  DefPrimitive!("\\textsevenoldstyle", "7", font => { family => "oldstyle" });
  DefPrimitive!("\\texteightoldstyle", "8", font => { family => "oldstyle" });
  DefPrimitive!("\\textnineoldstyle",  "9", font => { family => "oldstyle" });

  //======================================================================
  // Pair symbols
  DefPrimitive!("\\textlangle",     "\u{27E8}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textrangle",     "\u{27E9}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textlbrackdbl",  "\u{27E6}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textrbrackdbl",  "\u{27E7}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textuparrow",    "\u{2191}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textdownarrow",  "\u{2193}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textleftarrow",  "\u{2190}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textrightarrow", "\u{2192}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textlquill",     "\u{2045}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textrquill",     "\u{2046}",
    bounded => true, font => { encoding => "TS1" });

  //======================================================================
  // Monetary and Commercial symbols
  DefPrimitive!("\\textbaht",           "\u{0E3F}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textcent",           "\u{00A2}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textcentoldstyle",   "\u{00A2}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textcolonmonetary",  "\u{20A1}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textcurrency",       "\u{00A4}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textdollar",         "$",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textdollaroldstyle", "$",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textdong",           "\u{20AB}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\texteuro",           "\u{20AC}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textflorin",         "f",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textguarani",        "\u{20B2}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textlira",           "\u{20A4}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textnaira",          "\u{20A6}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textpeso",           "\u{20B1}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textsterling",       "\u{00A3}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textwon",            "\u{20A9}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textyen",            "\u{00A5}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textcircledP",       "\u{2117}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textcopyleft",       "\u{2184}\u{20DD}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textcopyright",      "\u{00A9}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textdiscount",       "\u{2052}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textestimated",      "\u{212E}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textpertenthousand", "\u{2031}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textperthousand",    "\u{2030}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textreferencemark",  "\u{203B}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textregistered",     "\u{00AE}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textservicemark",    "\u{2120}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\texttrademark",      "\u{2122}",
    bounded => true, font => { encoding => "TS1" });

  //======================================================================
  // Footnote symbols
  DefPrimitive!("\\textasteriskcentered", "*",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textbardbl",           "\u{2016}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textbrokenbar",        "\u{00A6}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textbullet",           "\u{2022}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textdagger",           "\u{2020}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textdaggerdbl",        "\u{2021}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textopenbullet",       "\u{25E6}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textparagraph",        "\u{00B6}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textperiodcentered",   "\u{00B7}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textpilcrow",          "\u{00B6}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textsection",          "\u{00A7}",
    bounded => true, font => { encoding => "TS1" });

  //======================================================================
  // Scientific symbols
  DefPrimitive!("\\textcelsius",      "\u{2103}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textdegree",       "\u{00B0}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textdiv",          "\u{00F7}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textlnot",         "\u{00AC}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textmho",          "\u{2127}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textminus",        "\u{2212}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textmu",           "\u{00B5}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textohm",          "\u{2126}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textordfeminine",  "\u{00AA}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textordmasculine", "\u{00BA}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textpm",           "\u{00B1}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textsurd",         "\u{221A}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\texttimes",        "\u{00D7}",
    bounded => true, font => { encoding => "TS1" });

  //======================================================================
  // Various
  DefPrimitive!("\\textacutedbl",             "\u{02DD}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textasciiacute",           "\u{00B4}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textasciibreve",           "\u{02D8}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textasciicaron",           "\u{02C7}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textasciidieresis",        "\u{00A8}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textasciigrave",           "\u{0060}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textasciimacron",          "\u{00AF}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textbigcircle",            "\u{25CB}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textblank",                "\u{2422}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textborn",                 "\u{2605}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textdblhyphen",            "=",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textdblhyphenchar",        "=",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textdied",                 "\u{271D}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textdivorced",             "\u{26AE}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textfractionsolidus",      "\u{002F}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textgravedbl",             "\u{201F}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textinterrobang",          "\u{203D}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textinterrobangdown",      "\u{2E18}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textleaf",                 "\u{1F342}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textmarried",              "\u{26AD}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textmusicalnote",          "\u{266A}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textnumero",               "\u{2116}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textquotesingle",          "'",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textquotestraightbase",    "\u{201A}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textquotestraightdblbase", "\u{201E}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textrecipe",               "\u{211E}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\textthreequartersemdash",  "\u{2014}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\texttildelow",             "~",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\texttwelveudash",          "\u{2014}",
    bounded => true, font => { encoding => "TS1" });

  //======================================================================
  TeX!(r"\DeclareFontEncoding{TS1}{}{}");
});
