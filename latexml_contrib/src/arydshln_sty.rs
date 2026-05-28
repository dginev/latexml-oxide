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
  def_macro_noop("\\ADLactivate")?;
  def_macro_noop("\\ADLdrawingmode")?;
  def_macro_noop("\\ADLinactivate")?;
  def_macro_noop("\\ADLnoshorthanded")?;
  def_macro_noop("\\ADLnullwide")?;
  def_macro_noop("\\ADLnullwidehline")?;
  def_macro_noop("\\ADLsomewide")?;
  def_macro_noop("\\ADLsomewidehline")?;
  def_macro_noop("\\arrayrulecolor")?;
  def_macro_noop("\\dashgapcolor{}")?;
  def_macro_noop("\\doublerulesepcolor")?;
  // NOTE: do NOT noop `\endlongtable`. The ar5iv Perl binding
  // (arydshln.sty.ltxml L45) does `DefMacro('\endlongtable', Tokens())`, but
  // that diverges from the REAL arydshln.sty, which SAVES and RESTORES
  // longtable's original `\endlongtable` (`\let\endlongtable\adl@org@endlongtable`,
  // arydshln.sty L796) rather than neutralizing it. Our longtable binding
  // relies on `\endlongtable` = `\lx@end@alignment\@end@tabular` to close the
  // alignment's boxing group; noop'ing it leaks that `{`-group so the
  // environment's `\endgroup` mismatches → mode cascade → `pop last locked
  // stack frame` FATAL (1510.04473: any `arydshln` + `longtable` with `p{}`
  // columns). Perl recovers from the same mismatch with 9 errors; our engine
  // aborts. Keeping longtable's `\endlongtable` functional matches the real
  // package and produces clean output (0 errors).
  def_macro_noop("\\nodashgapcolor")?;
  def_macro_noop("\\xleaders")?;
});
