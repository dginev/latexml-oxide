use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: sidecap.sty.ltxml
  // First draft stub; ignore sidecaption-ness and just make a regular table/figure
  RequirePackage!("ifthen");

  DefMacro!("\\SCtable[][]", "\\table[#2]");
  DefMacro!("\\endSCtable", "\\endtable");
  DefMacro!("\\csname SCtable*\\endcsname[][]", "\\csname table*\\endcsname[#2]");
  DefMacro!("\\csname endSCtable*\\endcsname", "\\csname endtable*\\endcsname");
  DefMacro!("\\SCfigure[][]", "\\figure[#2]");
  DefMacro!("\\endSCfigure", "\\endfigure");
  DefMacro!("\\csname SCfigure*\\endcsname[][]", "\\csname figure*\\endcsname[#2]");
  DefMacro!("\\csname endSCfigure*\\endcsname", "\\csname endfigure*\\endcsname");

  DefEnvironment!("{wide}", "#body");
});
