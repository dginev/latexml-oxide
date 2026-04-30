use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "arydshln.sty",
    "arydshln.sty is only minimally stubbed and will not be interpreted raw."
  );
  // TODO: Extend the internal Alignment machinery to facilitate a dashed bottom border directive
  Let!("\\hdashline", "\\hline");
  Let!("\\cdashline", "\\cline");
  // ar5iv-bindings/bindings/arydshln.sty.ltxml L21-24: ':' column type adds a
  // dashed right-border marker via the between-column slot. The \vrule gets
  // decorated by \@ADDCLASS{ltx_border_r_dashed}\relax so CSS renders a
  // dashed vertical rule.
  DefColumnType!(":", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_between_column(vec![
        T_CS!("\\vrule"),
        T_CS!("\\@ADDCLASS"),
        T_BEGIN!(),
        T_OTHER!("ltx_border_r_dashed"),
        T_END!(),
        T_CS!("\\relax"),
      ]);
    });
  });
  Let!("\\firsthdashline", "\\firsthline");
  Let!("\\lasthdashline", "\\lasthline");
  DefRegister!("\\dashlinedash" => Dimension!("4pt"));
  DefRegister!("\\dashlinegap" => Dimension!("4pt"));
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
