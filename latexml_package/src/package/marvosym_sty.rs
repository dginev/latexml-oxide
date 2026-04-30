use crate::prelude::*;

LoadDefinitions!({
  // Communication
  DefPrimitive!("\\Pickup", "\u{26AA}\u{0327}");
  DefPrimitive!("\\Letter", "\u{1F582}");
  DefPrimitive!("\\Mobilefone", "\u{1F4F1}");
  DefPrimitive!("\\Telefon", "\u{260E}");
  // Perl: DefMacro then DefPrimitive — the DefPrimitive overrides
  DefPrimitive!("\\fax", "FAX", bounded => true, font => {family => "sansserif", series => "bold"});
  DefMacro!("\\FAX", None, "\\lx@framed{\\fax}");
  DefPrimitive!("\\Fax", "\u{1F4E0}");
  DefPrimitive!("\\Faxmachine", "\u{1F4E0}");
  DefPrimitive!("\\Email", "\u{1F584}");
  DefPrimitive!("\\Lightning", "\u{21AF}");
  DefPrimitive!("\\EmailCT", "\u{2607}");
  Let!("\\Emailct", "\\EmailCT");

  // Engineering
  DefMacro!("\\Beam", None, "\\lx@nounicode{\\Beam}");
  DefPrimitive!("\\Bearing", "\u{25B5}\u{030A}");
  DefPrimitive!("\\LooseBearing", "\u{25B5}\u{030A}\u{0332}");
  Let!("\\Loosebearing", "\\LooseBearing");
  DefMacro!("\\FixedBearing", None, "\\lx@nounicode{\\FixedBearing}");
  Let!("\\Fixedbearing", "\\FixedBearing");
  DefPrimitive!("\\LeftTorque", "\u{2938}");
  Let!("\\Lefttorque", "\\LeftTorque");
  DefPrimitive!("\\RightTorque", "\u{2939}");
  Let!("\\Righttorque", "\\RightTorque");
  DefMacro!("\\Lineload", None, "\\lx@nounicode{\\Lineload}");
  DefPrimitive!("\\MVArrowDown", "\u{2193}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  Let!("\\Force", "\\MVArrowDown");

  DefMacro!("\\Octosteel", None, "\\lx@nounicode{\\Octosteel}");
  Let!("\\OktoSteel", "\\Octosteel");
  DefPrimitive!("\\HexaSteel", "\u{2B23}");
  Let!("\\Hexasteel", "\\HexaSteel");
  DefPrimitive!("\\SquareSteel", "\u{25FC}");
  Let!("\\Squaresteel", "\\SquareSteel");
  DefPrimitive!("\\RectSteel", "\u{25AE}");
  Let!("\\Rectsteel", "\\RectSteel");
  DefPrimitive!("\\Circsteel", "\u{26AB}");
  Let!("\\CircSteel", "\\Circsteel");
  DefPrimitive!("\\SquarePipe", "\u{25FB}");
  Let!("\\Squarepipe", "\\SquarePipe");
  DefPrimitive!("\\RectPipe", "\u{25AF}");
  Let!("\\Rectpipe", "\\RectPipe");
  DefPrimitive!("\\CircPipe", "\u{26AA}");
  Let!("\\Circpipe", "\\CircPipe");

  DefPrimitive!("\\lx@mvs@LSteel", "\u{2517}");
  DefPrimitive!("\\lx@mvs@RoundedLSteel", "\u{2514}");
  DefPrimitive!("\\lx@mvs@TSteel", "\u{2533}");
  DefPrimitive!("\\lx@mvs@RoundedTSteel", "\u{252C}");
  DefPrimitive!("\\lx@mvs@TTSteel@bottom", "\u{253B}");
  DefPrimitive!("\\lx@mvs@RoundedTTSteel@bottom", "\u{2534}");
  DefMacro!(
    "\\LSteel",
    None,
    "\\lx@tweaked{yoffset=-0.5ex}{\\lx@mvs@LSteel}"
  );
  DefMacro!(
    "\\RoundedLSteel",
    None,
    "\\lx@tweaked{yoffset=-0.5ex}{\\lx@mvs@RoundedLSteel}"
  );
  DefMacro!(
    "\\TSteel",
    None,
    "\\lx@tweaked{yoffset=0.5ex}{\\lx@mvs@TSteel}"
  );
  DefMacro!(
    "\\RoundedTSteel",
    None,
    "\\lx@tweaked{yoffset=0.5ex}{\\lx@mvs@RoundedTSteel}"
  );
  DefMacro!(
    "\\TTSteel",
    None,
    "\\lx@tweaked{yoffset=0.5ex}{\\lx@mvs@TSteel}\\lx@tweaked{xoffset=-0.6em,yoffset=-0.5ex}{\\lx@mvs@TTSteel@bottom}"
  );
  DefMacro!(
    "\\RoundedTTSteel",
    None,
    "\\lx@tweaked{yoffset=0.5ex}{\\lx@mvs@RoundedTSteel}\\lx@tweaked{xoffset=-0.6em,yoffset=-0.5ex}{\\lx@mvs@RoundedTTSteel@bottom}"
  );
  DefPrimitive!("\\FlatSteel", "\u{2501}");
  DefPrimitive!("\\Valve", "\u{25B6}\u{25C0}");
  Let!("\\Lsteel", "\\LSteel");
  Let!("\\RoundedLsteel", "\\RoundedLSteel");
  Let!("\\Tsteel", "\\TSteel");
  Let!("\\RoundedTsteel", "\\RoundedTSteel");
  Let!("\\TTsteel", "\\TTSteel");
  Let!("\\RoundedTTsteel", "\\RoundedTTSteel");
  Let!("\\Flatsteel", "\\FlatSteel");

  // Information
  DefMacro!("\\Industry", None, "\\lx@nounicode{\\Industry}");
  DefPrimitive!("\\Coffeecup", "\u{2615}");
  DefPrimitive!("\\LeftScissors", "\u{2702}");
  DefPrimitive!("\\CuttingLine", "\u{2504}");
  DefMacro!("\\RightScissors", None, "\\lx@hflipped{\\LeftScissors}");
  // Bizarre — yes, they're switched!
  Let!("\\Rightscissors", "\\LeftScissors");
  Let!("\\Leftscissors", "\\RightScissors");
  DefPrimitive!("\\Football", "\u{26BD}");
  DefPrimitive!("\\Bicycle", "\u{1F6B2}");

  DefPrimitive!("\\lx@mvs@Info", "\u{2139}");
  DefMacro!("\\Info", None, "\\lx@framed{\\lx@mvs@Info}");
  DefPrimitive!("\\ClockLogo", "\u{23F2}");
  Let!("\\Clocklogo", "\\ClockLogo");
  Let!("\\CutLine", "\\CuttingLine");
  Let!("\\Cutline", "\\CuttingLine");
  Let!("\\Kutline", "\\CuttingLine");

  DefPrimitive!("\\lx@mvs@CutLeft", "\u{2701}");
  DefMacro!(
    "\\CutLeft",
    None,
    "\\lx@mvs@CutLeft\\lx@tweaked{xoffset=-0.8em,yoffset=-0.4ex}{\\CutLine}"
  );
  DefMacro!("\\CutRight", None, "\\lx@hflipped{\\CutLeft}");
  // Bizarre — yes, they're switched!
  Let!("\\Cutright", "\\CutLeft");
  Let!("\\Cutleft", "\\CutRight");

  DefPrimitive!("\\Wheelchair", "\u{267F}");
  DefPrimitive!("\\Gentsroom", "\u{1F6B9}");
  DefPrimitive!("\\Ladiesroom", "\u{1F6BA}");

  DefPrimitive!("\\Checkedbox", "\u{2611}");
  DefPrimitive!("\\CrossedBox", "\u{2612}");
  Let!("\\Crossedbox", "\\CrossedBox");
  DefPrimitive!("\\HollowBox", "\u{2610}");
  DefPrimitive!("\\PointingHand", "\u{261E}");
  Let!("\\Pointinghand", "\\PointingHand");
  DefPrimitive!("\\WritingHand", "\u{270D}");
  Let!("\\Writinghand", "\\WritingHand");
  DefPrimitive!("\\MineSign", "\u{2692}");
  DefPrimitive!("\\Recycling", "\u{2672}");
  DefMacro!("\\PackingWaste", None, "\\lx@nounicode{\\PackingWaste}");

  // Laundry
  DefPrimitive!("\\lx@mvs@crossout", "\u{2573}");
  DefMacro!("\\WashCotton", None, "\\lx@nounicode{\\WashCotton}");
  DefMacro!("\\WashSynthetics", None, "\\lx@nounicode{\\WashSynthetics}");
  DefMacro!("\\WashWool", None, "\\lx@nounicode{\\WashWool}");
  DefMacro!("\\HandWash", None, "\\lx@nounicode{\\HandWash}");
  DefMacro!("\\Handwash", None, "\\lx@nounicode{\\Handwash}");
  DefMacro!("\\NoWash", None, "\\lx@nounicode{\\Handwash}");
  Let!("\\Dontwash", "\\NoWash");
  DefPrimitive!("\\Tumbler", "\u{29C7}");
  DefMacro!(
    "\\NoTumbler",
    None,
    "\\Tumbler\\lx@tweaked{xoffset=-0.8em}{\\lx@mvs@crossout}"
  );
  DefPrimitive!("\\lx@mvs@ChemicalCleaning", "\u{25EF}");
  DefMacro!(
    "\\NoChemicalCleaning",
    None,
    "\\lx@mvs@ChemicalCleaning\\lx@tweaked{xoffset=-0.8em}{\\lx@mvs@crossout}"
  );
  DefPrimitive!("\\Bleech", "\u{25B3}");
  DefMacro!(
    "\\NoBleech",
    None,
    "\\Bleech\\lx@tweaked{xoffset=-0.8em}{\\lx@mvs@crossout}"
  );
  DefPrimitive!("\\CleaningA", "\u{24B6}");
  DefPrimitive!("\\CleaningP", "\u{24C5}");
  DefMacro!("\\CleaningPP", None, "\\underline{\\CleaningP}");
  DefPrimitive!("\\CleaningF", "\u{24BB}");
  DefMacro!("\\CleaningFF", None, "\\underline{\\CleaningF}");

  DefMacro!("\\Ironing", None, "\\lx@nounicode{\\Ironing}");
  DefMacro!("\\ironing", None, "\\lx@nounicode{\\ironing}");
  DefMacro!("\\IRONING", None, "\\lx@nounicode{\\IRONING}");
  DefMacro!("\\IroningI", None, "\\lx@nounicode{\\IroningI}");
  DefMacro!("\\IroningII", None, "\\lx@nounicode{\\IroningII}");
  DefMacro!("\\IroningIII", None, "\\lx@nounicode{\\IroningIII}");
  DefMacro!("\\NoIroning", None, "\\lx@nounicode{\\NoIroning}");

  DefMacro!("\\AtNinetyFive", None, "\\lx@nounicode{\\AtNinetyFive}");
  DefMacro!(
    "\\ShortNinetyFive",
    None,
    "\\lx@nounicode{\\ShortNinetyFive}"
  );
  DefMacro!("\\AtSixty", None, "\\lx@nounicode{\\AtSixty}");
  DefMacro!("\\ShortSixty", None, "\\lx@nounicode{\\ShortSixty}");
  DefMacro!("\\ShortFifty", None, "\\lx@nounicode{\\ShortFifty}");
  DefMacro!("\\AtForty", None, "\\lx@nounicode{\\AtForty}");
  DefMacro!("\\ShortForty", None, "\\lx@nounicode{\\ShortForty}");
  DefMacro!("\\SpecialForty", None, "\\lx@nounicode{\\SpecialForty}");
  DefMacro!("\\ShortThirty", None, "\\lx@nounicode{\\ShortThirty}");

  // Currency
  DefPrimitive!("\\EUR",    "\u{20AC}",
    bounded => true, font => {family => "sansserif"});
  DefPrimitive!("\\EURdig", "\u{20AC}",
    bounded => true, font => {family => "sansserif"});
  DefPrimitive!("\\EURhv", "\u{20AC}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\EURcr", "\u{20AC}");
  DefPrimitive!("\\EURtm", "\u{20AC}",
    bounded => true, font => {series => "bold"});
  Let!("\\EurDig", "\\EURdig");
  Let!("\\EurHv", "\\EURhv");
  Let!("\\EurCr", "\\EURcr");
  Let!("\\EurTm", "\\EURtm");
  DefPrimitive!("\\Ecommerce", "\u{212E}");
  DefPrimitive!("\\EstimatedSign", "\u{212E}");
  DefPrimitive!("\\Shilling", "\u{00DF}");
  DefMacro!("\\Denarius", None, "\\lx@nounicode{\\Denarius}");
  DefMacro!("\\Deleatur", None, "\\lx@nounicode{\\Deleatur}");
  DefMacro!("\\Pfund", None, "\\lx@nounicode{\\Pfund}");
  DefPrimitive!("\\EyesDollar", "\u{1F4B2}");
  DefPrimitive!("\\Florin", "\u{0192}");

  // Safety
  DefMacro!("\\Stopsign", None, "\\lx@nounicode{\\Stopsign}");
  DefPrimitive!("\\CESign", "C\u{03F5}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  Let!("\\CEsign", "\\CESign");
  DefMacro!("\\Estatically", None, "\\lx@nounicode{\\Estatically}");
  DefMacro!("\\Explosionsafe", None, "\\lx@nounicode{\\Explosionsafe}");
  DefPrimitive!("\\lx@mvs@laser", "\u{2739}");
  DefMacro!(
    "\\Laserbeam",
    None,
    "\\lx@mvs@laser\\lx@tweaked{xoffset=-0.2em}{\\lx@emdash}"
  );
  DefPrimitive!("\\Biohazard", "\u{2623}");
  DefPrimitive!("\\Radioactivity", "\u{2622}");
  DefMacro!("\\BSEFree", None, "\\lx@nounicode{\\BSEFree}");
  DefMacro!("\\BSEfree", None, "\\lx@nounicode{\\BSEfree}");

  // Navigation
  DefPrimitive!("\\RewindToIndex", "|\u{25C0}");
  DefPrimitive!("\\RewindToStart", "|\u{25C0}\u{25C0}");
  DefPrimitive!("\\Rewind", "\u{25C0}");
  DefPrimitive!("\\Forward", "\u{25B6}");
  DefPrimitive!("\\ForwardToEnd", "\u{25B6}|");
  DefPrimitive!("\\ForwardToIndex", "\u{25B6}\u{25B6}|");
  DefPrimitive!("\\MoveUp", "\u{25B2}");
  DefPrimitive!("\\MoveDown", "\u{25BC}");
  DefPrimitive!("\\ToTop", "\u{25B2}\u{0305}");
  DefPrimitive!("\\ToBottom", "\u{25BC}\u{0332}");

  // Computers
  DefPrimitive!("\\ComputerMouse", "\u{1F5B0}");
  DefMacro!(
    "\\SerialInterface",
    None,
    "\\lx@nounicode{\\SerialInterface}"
  );
  DefPrimitive!("\\Keyboard", "\u{2328}");
  DefMacro!("\\SerialPort", None, "\\lx@nounicode{\\SerialPort}");
  DefMacro!("\\ParallelPort", None, "\\lx@nounicode{\\ParallelPort}");
  DefPrimitive!("\\Printer", "\u{1F5A8}");

  // Numbers
  DefPrimitive!("\\MVZero",  "0",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVOne",   "1",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVTwo",   "2",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVThree", "3",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVFour",  "4",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVFive",  "5",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVSix",   "6",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVSeven", "7",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVEight", "8",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVNine",  "9",
    bounded => true, font => {family => "sansserif", series => "bold"});

  // Maths
  DefPrimitive!("\\MVRightBracket", ")",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVLeftBracket", "(",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVComma",    ",",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVPeriod",   ".",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVMinus",    "-",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVPlus",     "+",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVDivision", "/",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MVMultiplication", "\u{00D7}",
    bounded => true, font => {family => "sansserif", series => "bold"});

  DefPrimitive!("\\Conclusion", "\u{21D2}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\Equivalence", "\u{21D4}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\barOver", "\u{203E}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\BarOver", "\u{203E}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\lx@mvs@rightarrow", "\u{2192}");
  DefMacro!(
    "\\arrowOver",
    None,
    "\\lx@tweaked{yoffset=0.5ex}{\\lx@mvs@rightarrow}"
  );
  DefMacro!(
    "\\ArrowOver",
    None,
    "\\lx@tweaked{yoffset=0.7ex}{\\lx@mvs@rightarrow}"
  );
  Let!("\\Vectorarrow", "\\arrowOver");
  Let!("\\Vectorarrowhigh", "\\ArrowOver");
  // Perl: DefMacro then DefPrimitive — the DefPrimitive overrides
  DefPrimitive!("\\StrikingThrough", "/",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\MultiplicationDot", "\u{00B7}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  Let!("\\Squaredot", "\\MultiplicationDot");

  DefPrimitive!("\\LessOrEqual", "\u{2264}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\LargerOrEqual", "\u{2265}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\AngleSign", "\u{2222}");
  Let!("\\Anglesign", "\\AngleSign");
  DefPrimitive!("\\Corresponds", "\u{2259}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\Congruent", "\u{2261}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\NotCongruent", "\u{2262}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\Divides", "\u{2044}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\DividesNot", "\u{2044}\u{20D2}",
    bounded => true, font => {family => "sansserif", series => "bold"});

  // Biology
  DefPrimitive!("\\Female", "\u{2640}");
  DefPrimitive!("\\Male", "\u{2642}");
  DefPrimitive!("\\Hermaphrodite", "\u{26A5}");
  DefPrimitive!("\\Neutral", "\u{26AC}");
  DefPrimitive!("\\FEMALE", "\u{2640}");
  DefPrimitive!("\\MALE", "\u{2642}");
  DefPrimitive!("\\HERMAPHRODITE", "\u{26A5}");
  DefPrimitive!("\\FemaleFemale", "\u{26A2}");
  DefPrimitive!("\\MaleMale", "\u{26A3}");
  DefPrimitive!("\\FemaleMale", "\u{26A4}");

  // Astronomy
  DefPrimitive!("\\Sun", "\u{2609}");
  DefPrimitive!("\\Moon", "\u{263D}");
  DefPrimitive!("\\Mercury", "\u{263F}");
  DefPrimitive!("\\Venus", "\u{2640}");
  DefPrimitive!("\\Mars", "\u{2642}");
  DefPrimitive!("\\Jupiter", "\u{2643}");
  DefPrimitive!("\\Saturn", "\u{2644}");
  DefPrimitive!("\\Uranus", "\u{2645}");
  DefPrimitive!("\\Neptune", "\u{2646}");
  DefPrimitive!("\\Pluto", "\u{2647}");
  DefPrimitive!("\\Earth", "\u{2641}");

  // Astrology
  DefPrimitive!("\\Aries", "\u{2648}");
  DefPrimitive!("\\Taurus", "\u{2649}");
  DefPrimitive!("\\Gemini", "\u{264A}");
  DefPrimitive!("\\Cancer", "\u{264B}");
  DefPrimitive!("\\Leo", "\u{264C}");
  DefPrimitive!("\\Virgo", "\u{264D}");
  DefPrimitive!("\\Libra", "\u{264E}");
  DefPrimitive!("\\Scorpio", "\u{264F}");
  DefPrimitive!("\\Sagittarius", "\u{2650}");
  DefPrimitive!("\\Capricorn", "\u{2651}");
  DefPrimitive!("\\Aquarius", "\u{2652}");
  DefPrimitive!("\\Pisces", "\u{2653}");

  DefMacro!(
    "\\Zodiac{}",
    "\\ifcase#1\\or\\Aries\\or\\Taurus\\or\\Gemini\\or\\Cancer\\or\\Leo\\or\\Virgo\\or\\Libra\\or\\Scorpio\\or\\Sagittarius\\or\\Capricorn\\or\\Aquarius\\or\\Pisces\\else???\\fi"
  );

  // Others
  DefPrimitive!("\\YinYang", "\u{262F}");
  Let!("\\Yinyang", "\\YinYang");
  Let!("\\Yingyang", "\\YinYang");
  DefPrimitive!("\\MVRightArrow", "\u{2192}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  Let!("\\MVRightarrow", "\\MVRightArrow");
  DefPrimitive!("\\MVAt",   "@",
    bounded => true, font => {family => "sansserif", series => "bold"});
  DefPrimitive!("\\BOLogo", "BO",
    bounded => true, font => {family => "sansserif", series => "bold"});
  Let!("\\BOLogoL", "\\BOLogo");
  Let!("\\BOLogoP", "\\BOLogo");
  Let!("\\FHBOlogo", "\\BOLogo");
  DefPrimitive!("\\Mundus", "\u{1F30D}");
  DefPrimitive!("\\Cross", "\u{2020}");
  DefPrimitive!("\\CeltCross", "\u{1F548}");
  Let!("\\Celtcross", "\\CeltCross");
  DefPrimitive!("\\Ankh", "\u{2625}");

  DefPrimitive!("\\Heart", "\u{2661}");
  DefPrimitive!("\\CircledA", "\u{24B6}");
  DefPrimitive!("\\Bouquet", "\u{1F395}");
  DefPrimitive!("\\Frowny", "\u{2639}");
  DefPrimitive!("\\Smiley", "\u{263A}");
  DefPrimitive!("\\PeaceDove", "\u{1F54A}");
  DefMacro!("\\Bat", None, "\\lx@nounicode{\\Bat}");
  DefPrimitive!("\\WomanFace", "\u{1F469}");
  Let!("\\Womanface", "\\WomanFace");
  DefPrimitive!("\\ManFace", "\u{1F468}");
  Let!("\\MartinVogel", "\\ManFace");

  // Low-level font accessors — stubs
  DefMacro!("\\mvchr", None, "\\lx@nounicode{\\mvchr}");
  DefMacro!("\\mvs", None, "\\lx@nounicode{\\mvs}");
  DefMacro!("\\textmvs", None, "\\lx@nounicode{\\textmvs}");
});
