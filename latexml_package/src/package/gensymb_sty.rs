use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\degree",     "\\ifmmode\\lx@math@degree\\else\\lx@text@degree\\fi");
  DefMacro!("\\celcius",    "\\ifmmode\\lx@math@celcius\\else\\lx@text@celcius\\fi");
  DefMacro!("\\perthousand","\\ifmmode\\lx@math@perthou\\else\\lx@text@perthou\\fi");
  DefMacro!("\\ohm",        "\\ifmmode\\lx@math@ohm\\else\\lx@text@ohm\\fi");
  DefMacro!("\\micro",      "\\ifmmode\\lx@math@micro\\else\\lx@text@micro\\fi");

  DefPrimitive!("\\lx@text@degree",  "\u{00B0}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\lx@text@celcius", "\u{2103}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\lx@text@perthou", "\u{2030}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\lx@text@ohm",     "\u{2126}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\lx@text@micro",   "\u{00B5}",
    bounded => true, font => { encoding => "TS1" });

  DefMath!("\\lx@math@degree", None, "\u{00B0}",
    bounded => true, font => { encoding => "TS1" }, alias => "\\degree");
  DefMath!("\\lx@math@celcius", None, "\u{2103}",
    bounded => true, font => { encoding => "TS1" }, alias => "\\celcius");
  DefMath!("\\lx@math@perthou", None, "\u{2030}",
    bounded => true, font => { encoding => "TS1" }, alias => "\\perthousand");
  DefMath!("\\lx@math@ohm", None, "\u{2126}",
    bounded => true, font => { encoding => "TS1" }, alias => "\\ohm");
  DefMath!("\\lx@math@micro", None, "\u{00B5}",
    bounded => true, font => { encoding => "TS1" }, alias => "\\micro");
});
