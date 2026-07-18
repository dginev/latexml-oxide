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

  // \sidecaptionvpos{<float type>}{<t|c|b>} — real sidecap.sty:
  // `\newcommand*\sidecaptionvpos[2]` — configures the vertical alignment of a
  // side caption. Purely a layout hint with no bearing on the logical HTML
  // output (we already ignore side-caption-ness above), so consume both args
  // and expand to nothing. Witness 2408.08435 (ar5iv #555).
  DefMacro!("\\sidecaptionvpos{}{}", "");

  DefEnvironment!("{wide}", "#body");
});
