use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "tabu.sty",
    "tabu.sty is only minimally stubbed and will not be interpreted raw."
  );
  RequirePackage!("array");
  RequirePackage!("varwidth");
  RequirePackage!("longtable");
  DefMacro!("\\tabu", "\\tabular");
  DefMacro!("\\endtabu", "\\endtabular");
  DefMacro!("\\longtabu", "\\longtable");
  DefMacro!("\\endlongtabu", "\\endlongtable");
  // stubs
  DefMacro!("\\savetabu{}", "");
  DefMacro!("\\usetabu{}", "");
  DefMacro!("\\preamble{}", "");
  DefMacro!("\\tabulinestyle{}", "");
  DefMacro!("\\newtabulinestyle{}", "");
  DefMacro!("\\tabucline[]{}", "\\hline");
  DefMacro!("\\taburulecolor OptionalMatch:| OptionalUntil:| {}", "");
  DefMacro!("\\taburowcolors[] Number {}", "");
  DefMacro!("\\tabuphantomline", "");
  DefRegister!("\\tracingtabu" => Number::new(0));
  DefRegister!("\\tabulinesep" => Dimension::new(0));
  DefRegister!("\\abovetabulinesep" => Dimension::new(0));
  DefRegister!("\\belowtabulinesep" => Dimension::new(0));
});
