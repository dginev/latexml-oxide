use crate::prelude::*;

/// DEP-NEW (2026-05-19): data-drive helper for the 113 marvosym
/// icon primitives that all expand to `DefPrimitive!("\\<cs>",
/// "<unicode-char(s)>")` — the same simple-literal-body shape the
/// `DefPrimitive!` macro's first arm expands. Inlining the body of
/// that arm into a runtime fn collapses 113 macro expansions to
/// one. Per [[wisdom_data_drive_min_call_sites]]: 113 ≫ 5
/// threshold. Matches the fontawesome DEP-15 / jhep_cls /
/// iopart_support_sty pattern.
fn def_marvosym_icon(cs: &str, codepoint: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(cs, true)?;
  let body_sym = arena::pin(codepoint);
  let cs_for_closure = cs_tok;
  let closure: PrimitiveBody = PrimitiveBody::Closure(std::rc::Rc::new(
    move |_args: Vec<ArgWrap>| {
      Tbox::new(
        body_sym,
        None,
        None,
        Tokens!(cs_for_closure),
        SymHashMap::default(),
      )
      .into_digested_result()
    },
  ));
  def_primitive(cs_tok, params, Some(closure), PrimitiveOptions::default())?;
  Ok(())
}

LoadDefinitions!({
  // Communication
  def_marvosym_icon("\\Pickup", "\u{26AA}\u{0327}")?;
  def_marvosym_icon("\\Letter", "\u{1F582}")?;
  def_marvosym_icon("\\Mobilefone", "\u{1F4F1}")?;
  def_marvosym_icon("\\Telefon", "\u{260E}")?;
  // Perl: DefMacro then DefPrimitive — the DefPrimitive overrides
  DefPrimitive!("\\fax", "FAX", bounded => true, font => {family => "sansserif", series => "bold"});
  DefMacro!("\\FAX", None, "\\lx@framed{\\fax}");
  def_marvosym_icon("\\Fax", "\u{1F4E0}")?;
  def_marvosym_icon("\\Faxmachine", "\u{1F4E0}")?;
  def_marvosym_icon("\\Email", "\u{1F584}")?;
  def_marvosym_icon("\\Lightning", "\u{21AF}")?;
  def_marvosym_icon("\\EmailCT", "\u{2607}")?;
  Let!("\\Emailct", "\\EmailCT");

  // Engineering
  DefMacro!("\\Beam", None, "\\lx@nounicode{\\Beam}");
  def_marvosym_icon("\\Bearing", "\u{25B5}\u{030A}")?;
  def_marvosym_icon("\\LooseBearing", "\u{25B5}\u{030A}\u{0332}")?;
  Let!("\\Loosebearing", "\\LooseBearing");
  DefMacro!("\\FixedBearing", None, "\\lx@nounicode{\\FixedBearing}");
  Let!("\\Fixedbearing", "\\FixedBearing");
  def_marvosym_icon("\\LeftTorque", "\u{2938}")?;
  Let!("\\Lefttorque", "\\LeftTorque");
  def_marvosym_icon("\\RightTorque", "\u{2939}")?;
  Let!("\\Righttorque", "\\RightTorque");
  DefMacro!("\\Lineload", None, "\\lx@nounicode{\\Lineload}");
  DefPrimitive!("\\MVArrowDown", "\u{2193}",
    bounded => true, font => {family => "sansserif", series => "bold"});
  Let!("\\Force", "\\MVArrowDown");

  DefMacro!("\\Octosteel", None, "\\lx@nounicode{\\Octosteel}");
  Let!("\\OktoSteel", "\\Octosteel");
  def_marvosym_icon("\\HexaSteel", "\u{2B23}")?;
  Let!("\\Hexasteel", "\\HexaSteel");
  def_marvosym_icon("\\SquareSteel", "\u{25FC}")?;
  Let!("\\Squaresteel", "\\SquareSteel");
  def_marvosym_icon("\\RectSteel", "\u{25AE}")?;
  Let!("\\Rectsteel", "\\RectSteel");
  def_marvosym_icon("\\Circsteel", "\u{26AB}")?;
  Let!("\\CircSteel", "\\Circsteel");
  def_marvosym_icon("\\SquarePipe", "\u{25FB}")?;
  Let!("\\Squarepipe", "\\SquarePipe");
  def_marvosym_icon("\\RectPipe", "\u{25AF}")?;
  Let!("\\Rectpipe", "\\RectPipe");
  def_marvosym_icon("\\CircPipe", "\u{26AA}")?;
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
  def_marvosym_icon("\\FlatSteel", "\u{2501}")?;
  def_marvosym_icon("\\Valve", "\u{25B6}\u{25C0}")?;
  Let!("\\Lsteel", "\\LSteel");
  Let!("\\RoundedLsteel", "\\RoundedLSteel");
  Let!("\\Tsteel", "\\TSteel");
  Let!("\\RoundedTsteel", "\\RoundedTSteel");
  Let!("\\TTsteel", "\\TTSteel");
  Let!("\\RoundedTTsteel", "\\RoundedTTSteel");
  Let!("\\Flatsteel", "\\FlatSteel");

  // Information
  DefMacro!("\\Industry", None, "\\lx@nounicode{\\Industry}");
  def_marvosym_icon("\\Coffeecup", "\u{2615}")?;
  def_marvosym_icon("\\LeftScissors", "\u{2702}")?;
  def_marvosym_icon("\\CuttingLine", "\u{2504}")?;
  DefMacro!("\\RightScissors", None, "\\lx@hflipped{\\LeftScissors}");
  // Bizarre — yes, they're switched!
  Let!("\\Rightscissors", "\\LeftScissors");
  Let!("\\Leftscissors", "\\RightScissors");
  def_marvosym_icon("\\Football", "\u{26BD}")?;
  def_marvosym_icon("\\Bicycle", "\u{1F6B2}")?;

  DefPrimitive!("\\lx@mvs@Info", "\u{2139}");
  DefMacro!("\\Info", None, "\\lx@framed{\\lx@mvs@Info}");
  def_marvosym_icon("\\ClockLogo", "\u{23F2}")?;
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

  def_marvosym_icon("\\Wheelchair", "\u{267F}")?;
  def_marvosym_icon("\\Gentsroom", "\u{1F6B9}")?;
  def_marvosym_icon("\\Ladiesroom", "\u{1F6BA}")?;

  def_marvosym_icon("\\Checkedbox", "\u{2611}")?;
  def_marvosym_icon("\\CrossedBox", "\u{2612}")?;
  Let!("\\Crossedbox", "\\CrossedBox");
  def_marvosym_icon("\\HollowBox", "\u{2610}")?;
  def_marvosym_icon("\\PointingHand", "\u{261E}")?;
  Let!("\\Pointinghand", "\\PointingHand");
  def_marvosym_icon("\\WritingHand", "\u{270D}")?;
  Let!("\\Writinghand", "\\WritingHand");
  def_marvosym_icon("\\MineSign", "\u{2692}")?;
  def_marvosym_icon("\\Recycling", "\u{2672}")?;
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
  def_marvosym_icon("\\Tumbler", "\u{29C7}")?;
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
  def_marvosym_icon("\\Bleech", "\u{25B3}")?;
  DefMacro!(
    "\\NoBleech",
    None,
    "\\Bleech\\lx@tweaked{xoffset=-0.8em}{\\lx@mvs@crossout}"
  );
  def_marvosym_icon("\\CleaningA", "\u{24B6}")?;
  def_marvosym_icon("\\CleaningP", "\u{24C5}")?;
  DefMacro!("\\CleaningPP", None, "\\underline{\\CleaningP}");
  def_marvosym_icon("\\CleaningF", "\u{24BB}")?;
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
  def_marvosym_icon("\\EURcr", "\u{20AC}")?;
  DefPrimitive!("\\EURtm", "\u{20AC}",
    bounded => true, font => {series => "bold"});
  Let!("\\EurDig", "\\EURdig");
  Let!("\\EurHv", "\\EURhv");
  Let!("\\EurCr", "\\EURcr");
  Let!("\\EurTm", "\\EURtm");
  def_marvosym_icon("\\Ecommerce", "\u{212E}")?;
  def_marvosym_icon("\\EstimatedSign", "\u{212E}")?;
  def_marvosym_icon("\\Shilling", "\u{00DF}")?;
  DefMacro!("\\Denarius", None, "\\lx@nounicode{\\Denarius}");
  DefMacro!("\\Deleatur", None, "\\lx@nounicode{\\Deleatur}");
  DefMacro!("\\Pfund", None, "\\lx@nounicode{\\Pfund}");
  def_marvosym_icon("\\EyesDollar", "\u{1F4B2}")?;
  def_marvosym_icon("\\Florin", "\u{0192}")?;

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
  def_marvosym_icon("\\Biohazard", "\u{2623}")?;
  def_marvosym_icon("\\Radioactivity", "\u{2622}")?;
  DefMacro!("\\BSEFree", None, "\\lx@nounicode{\\BSEFree}");
  DefMacro!("\\BSEfree", None, "\\lx@nounicode{\\BSEfree}");

  // Navigation
  DefPrimitive!("\\RewindToIndex", "|\u{25C0}");
  DefPrimitive!("\\RewindToStart", "|\u{25C0}\u{25C0}");
  def_marvosym_icon("\\Rewind", "\u{25C0}")?;
  def_marvosym_icon("\\Forward", "\u{25B6}")?;
  DefPrimitive!("\\ForwardToEnd", "\u{25B6}|");
  DefPrimitive!("\\ForwardToIndex", "\u{25B6}\u{25B6}|");
  def_marvosym_icon("\\MoveUp", "\u{25B2}")?;
  def_marvosym_icon("\\MoveDown", "\u{25BC}")?;
  def_marvosym_icon("\\ToTop", "\u{25B2}\u{0305}")?;
  def_marvosym_icon("\\ToBottom", "\u{25BC}\u{0332}")?;

  // Computers
  def_marvosym_icon("\\ComputerMouse", "\u{1F5B0}")?;
  DefMacro!(
    "\\SerialInterface",
    None,
    "\\lx@nounicode{\\SerialInterface}"
  );
  def_marvosym_icon("\\Keyboard", "\u{2328}")?;
  DefMacro!("\\SerialPort", None, "\\lx@nounicode{\\SerialPort}");
  DefMacro!("\\ParallelPort", None, "\\lx@nounicode{\\ParallelPort}");
  def_marvosym_icon("\\Printer", "\u{1F5A8}")?;

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
  def_marvosym_icon("\\AngleSign", "\u{2222}")?;
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
  def_marvosym_icon("\\Female", "\u{2640}")?;
  def_marvosym_icon("\\Male", "\u{2642}")?;
  def_marvosym_icon("\\Hermaphrodite", "\u{26A5}")?;
  def_marvosym_icon("\\Neutral", "\u{26AC}")?;
  def_marvosym_icon("\\FEMALE", "\u{2640}")?;
  def_marvosym_icon("\\MALE", "\u{2642}")?;
  def_marvosym_icon("\\HERMAPHRODITE", "\u{26A5}")?;
  def_marvosym_icon("\\FemaleFemale", "\u{26A2}")?;
  def_marvosym_icon("\\MaleMale", "\u{26A3}")?;
  def_marvosym_icon("\\FemaleMale", "\u{26A4}")?;

  // Astronomy
  def_marvosym_icon("\\Sun", "\u{2609}")?;
  def_marvosym_icon("\\Moon", "\u{263D}")?;
  def_marvosym_icon("\\Mercury", "\u{263F}")?;
  def_marvosym_icon("\\Venus", "\u{2640}")?;
  def_marvosym_icon("\\Mars", "\u{2642}")?;
  def_marvosym_icon("\\Jupiter", "\u{2643}")?;
  def_marvosym_icon("\\Saturn", "\u{2644}")?;
  def_marvosym_icon("\\Uranus", "\u{2645}")?;
  def_marvosym_icon("\\Neptune", "\u{2646}")?;
  def_marvosym_icon("\\Pluto", "\u{2647}")?;
  def_marvosym_icon("\\Earth", "\u{2641}")?;

  // Astrology
  def_marvosym_icon("\\Aries", "\u{2648}")?;
  def_marvosym_icon("\\Taurus", "\u{2649}")?;
  def_marvosym_icon("\\Gemini", "\u{264A}")?;
  def_marvosym_icon("\\Cancer", "\u{264B}")?;
  def_marvosym_icon("\\Leo", "\u{264C}")?;
  def_marvosym_icon("\\Virgo", "\u{264D}")?;
  def_marvosym_icon("\\Libra", "\u{264E}")?;
  def_marvosym_icon("\\Scorpio", "\u{264F}")?;
  def_marvosym_icon("\\Sagittarius", "\u{2650}")?;
  def_marvosym_icon("\\Capricorn", "\u{2651}")?;
  def_marvosym_icon("\\Aquarius", "\u{2652}")?;
  def_marvosym_icon("\\Pisces", "\u{2653}")?;

  DefMacro!(
    "\\Zodiac{}",
    "\\ifcase#1\\or\\Aries\\or\\Taurus\\or\\Gemini\\or\\Cancer\\or\\Leo\\or\\Virgo\\or\\Libra\\or\\Scorpio\\or\\Sagittarius\\or\\Capricorn\\or\\Aquarius\\or\\Pisces\\else???\\fi"
  );

  // Others
  def_marvosym_icon("\\YinYang", "\u{262F}")?;
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
  def_marvosym_icon("\\Mundus", "\u{1F30D}")?;
  def_marvosym_icon("\\Cross", "\u{2020}")?;
  def_marvosym_icon("\\CeltCross", "\u{1F548}")?;
  Let!("\\Celtcross", "\\CeltCross");
  def_marvosym_icon("\\Ankh", "\u{2625}")?;

  def_marvosym_icon("\\Heart", "\u{2661}")?;
  def_marvosym_icon("\\CircledA", "\u{24B6}")?;
  def_marvosym_icon("\\Bouquet", "\u{1F395}")?;
  def_marvosym_icon("\\Frowny", "\u{2639}")?;
  def_marvosym_icon("\\Smiley", "\u{263A}")?;
  def_marvosym_icon("\\PeaceDove", "\u{1F54A}")?;
  DefMacro!("\\Bat", None, "\\lx@nounicode{\\Bat}");
  def_marvosym_icon("\\WomanFace", "\u{1F469}")?;
  Let!("\\Womanface", "\\WomanFace");
  def_marvosym_icon("\\ManFace", "\u{1F468}")?;
  Let!("\\MartinVogel", "\\ManFace");

  // Low-level font accessors — stubs
  DefMacro!("\\mvchr", None, "\\lx@nounicode{\\mvchr}");
  DefMacro!("\\mvs", None, "\\lx@nounicode{\\mvs}");
  DefMacro!("\\textmvs", None, "\\lx@nounicode{\\textmvs}");
});
