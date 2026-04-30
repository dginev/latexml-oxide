use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl gensymb.sty.ltxml L19-23 passes `protected => 1` so these CSes
  // don't expand under \edef/\write contexts. Rust was missing the flag.
  DefMacro!("\\degree",     None, "\\ifmmode\\lx@math@degree\\else\\lx@text@degree\\fi",
    protected => true);
  DefMacro!("\\celcius",    None, "\\ifmmode\\lx@math@celcius\\else\\lx@text@celcius\\fi",
    protected => true);
  DefMacro!("\\perthousand", None, "\\ifmmode\\lx@math@perthou\\else\\lx@text@perthou\\fi",
    protected => true);
  DefMacro!("\\ohm",        None, "\\ifmmode\\lx@math@ohm\\else\\lx@text@ohm\\fi",
    protected => true);
  DefMacro!("\\micro",      None, "\\ifmmode\\lx@math@micro\\else\\lx@text@micro\\fi",
    protected => true);

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
