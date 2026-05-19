use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

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
  def_macro_noop("\\endlongtable")?;
  def_macro_noop("\\nodashgapcolor")?;
  def_macro_noop("\\xleaders")?;
});
