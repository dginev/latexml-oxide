use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl gensymb.sty.ltxml L19-23 passes `protected => 1` so these CSes
  // don't expand under \edef/\write contexts. Rust was missing the flag.
  DefMacro!("\\lx@gns@degree",     None, "\\ifmmode\\lx@math@degree\\else\\lx@text@degree\\fi",
    protected => true);
  DefMacro!("\\lx@gns@celcius",    None, "\\ifmmode\\lx@math@celcius\\else\\lx@text@celcius\\fi",
    protected => true);
  // gensymb.sty:38/64 names it `\celsius`; Perl's binding spells it `\celcius` (KNOWN_PERL_ERRORS #635), which
  // we keep beside it. `\celsius` was the first error of 214 run-340 papers (witnesses 2604.04150, 2508.00245).
  DefMacro!("\\lx@gns@celsius",    None, "\\ifmmode\\lx@math@celsius\\else\\lx@text@celsius\\fi",
    protected => true);
  DefMacro!("\\lx@gns@perthousand", None, "\\ifmmode\\lx@math@perthou\\else\\lx@text@perthou\\fi",
    protected => true);
  DefMacro!("\\lx@gns@ohm",        None, "\\ifmmode\\lx@math@ohm\\else\\lx@text@ohm\\fi",
    protected => true);
  DefMacro!("\\lx@gns@micro",      None, "\\ifmmode\\lx@math@micro\\else\\lx@text@micro\\fi",
    protected => true);

  // gensymb.sty:112-140 defines its commands in `\AtBeginDocument` (`\gns@usetcsymbols`/`\gns@usefakedsymbols`, …),
  // over a document's own preamble definition: 2206.05542's main.tex:54-58 restores a one-argument `\degree` after
  // loading gensymb, which TeX's body `\degree` no longer has. Set at load too (gensymb.sty:33-35 defines `\degree`,
  // `\celsius`, `\ohm` there).
  RawTeX!(r"\let\degree\lx@gns@degree\let\celcius\lx@gns@celcius\let\celsius\lx@gns@celsius\let\perthousand\lx@gns@perthousand\let\ohm\lx@gns@ohm\let\micro\lx@gns@micro\AtBeginDocument{\let\degree\lx@gns@degree\let\celcius\lx@gns@celcius\let\celsius\lx@gns@celsius\let\perthousand\lx@gns@perthousand\let\ohm\lx@gns@ohm\let\micro\lx@gns@micro}");
  DefPrimitive!("\\lx@text@degree",  "\u{00B0}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\lx@text@celcius", "\u{2103}",
    bounded => true, font => { encoding => "TS1" });
  DefPrimitive!("\\lx@text@celsius", "\u{2103}",
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
  DefMath!("\\lx@math@celsius", None, "\u{2103}",
    bounded => true, font => { encoding => "TS1" }, alias => "\\celsius");
  DefMath!("\\lx@math@perthou", None, "\u{2030}",
    bounded => true, font => { encoding => "TS1" }, alias => "\\perthousand");
  DefMath!("\\lx@math@ohm", None, "\u{2126}",
    bounded => true, font => { encoding => "TS1" }, alias => "\\ohm");
  DefMath!("\\lx@math@micro", None, "\u{00B5}",
    bounded => true, font => { encoding => "TS1" }, alias => "\\micro");
});
