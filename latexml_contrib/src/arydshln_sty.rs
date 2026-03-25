use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!("missing_file", "arydshln.sty",
    "arydshln.sty is only minimally stubbed and will not be interpreted raw.");
  // TODO: Extend the internal Alignment machinery to facilitate a dashed bottom border directive
  Let!("\\hdashline", "\\hline");
  Let!("\\cdashline", "\\cline");
  // TODO: Perl defines a ':' column type that adds a dashed vertical rule.
  // DefColumnType(':') not yet ported here.
  Let!("\\firsthdashline", "\\firsthline");
  Let!("\\lasthdashline", "\\lasthline");
  DefRegister!("\\dashlinedash" => Dimension::new_scaled(4 * 65536));   // 4pt
  DefRegister!("\\dashlinegap" => Dimension::new_scaled(4 * 65536));    // 4pt
  Let!("\\hdashlinewidth", "\\dashlinedash");
  Let!("\\hdashlinegap", "\\dashlinegap");
  DefMacro!("\\ADLactivate", "");
  DefMacro!("\\ADLdrawingmode", "");
  DefMacro!("\\ADLinactivate", "");
  DefMacro!("\\ADLnoshorthanded", "");
  DefMacro!("\\ADLnullwide", "");
  DefMacro!("\\ADLnullwidehline", "");
  DefMacro!("\\ADLsomewide", "");
  DefMacro!("\\ADLsomewidehline", "");
  DefMacro!("\\arrayrulecolor", "");
  DefMacro!("\\dashgapcolor{}", "");
  DefMacro!("\\doublerulesepcolor", "");
  DefMacro!("\\endlongtable", "");
  DefMacro!("\\nodashgapcolor", "");
  DefMacro!("\\xleaders", "");
});
