use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "memoir.cls",
    "memoir.cls is only minimally stubbed and will not be interpreted raw."
  );
  LoadClass!("OmniBus");
  RequirePackage!("iftex");
  RequirePackage!("array");
  RequirePackage!("dcolumn");
  RequirePackage!("tabularx");
  RequirePackage!("textcase");
  // These are originally \EmulatedPackage directives
  RequirePackage!("appendix");
  RequirePackage!("booktabs");
  RequirePackage!("changepage");
  RequirePackage!("chngcntr");
  RequirePackage!("chngpage");
  RequirePackage!("crop");
  RequirePackage!("enumerate");
  RequirePackage!("epigraph");
  RequirePackage!("makeidx");
  RequirePackage!("needspace");
  RequirePackage!("parskip");
  RequirePackage!("setspace");
  RequirePackage!("titling");
  RequirePackage!("tocbibind");
  RequirePackage!("verbatim");
});
