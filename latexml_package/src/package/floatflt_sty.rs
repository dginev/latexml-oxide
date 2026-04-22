use crate::prelude::*;

/// Perl floatflt.sty.ltxml L40-42: float direction from optional arg or
/// `floatfltpos` state: `v` / `r` prefix → right, else left. Prior Rust
/// hardcoded `float='right'`, ignoring both the option and `[l]` / `[p]`
/// optional-arg forms.
fn floatflt_float_direction(whatsit: &Whatsit) -> &'static str {
  let pos_arg = whatsit.get_arg(1).map(|a| a.to_string()).unwrap_or_default();
  let pos_arg = pos_arg.trim().to_string();
  let pos = if !pos_arg.is_empty() {
    pos_arg
  } else if let Some(Stored::String(sym)) = state::lookup_value("floatfltpos") {
    arena::with(sym, |s| s.to_string())
  } else {
    "v".to_string()
  };
  if pos.starts_with('v') || pos.starts_with('r') { "right" } else { "left" }
}

#[rustfmt::skip]
LoadDefinitions!({
  state::assign_value("floatfltpos", Stored::from("v"), None);
  DeclareOption!("rflt", { state::assign_value("floatfltpos", Stored::from("r"), None); });
  DeclareOption!("lflt", { state::assign_value("floatfltpos", Stored::from("l"), None); });
  DeclareOption!("pflt", { state::assign_value("floatfltpos", Stored::from("p"), None); });
  DeclareOption!("vflt", { state::assign_value("floatfltpos", Stored::from("v"), None); });
  // Use `after_digest` (runs while env frame is still active), NOT
  // `after_digest_body` which runs after the frame pops — `\@captype` set
  // by `before_float` via local-scope def_macro is gone by that point,
  // causing `after_float`'s `digest(\@captype)` to error with "T_CS[\@captype]
  // is not defined" (sandbox paper 0810.1610). The engine's `{figure}` /
  // `{table}` envs use `after_digest` for this reason; match them.
  DefEnvironment!("{floatingfigure}[]{Dimension}",
    "<ltx:figure xml:id='#id' inlist='#inlist' float='#float'>#tags #body</ltx:figure>",
    before_digest => {
      crate::engine::latex_constructs::before_float("figure", None);
    },
    after_digest => sub[whatsit] {
      whatsit.set_property("float", floatflt_float_direction(whatsit));
      crate::engine::latex_constructs::after_float(whatsit);
    });
  DefEnvironment!("{floatingtable}[]{Dimension}",
    "<ltx:table xml:id='#id' inlist='#inlist' float='#float'>#tags #body</ltx:table>",
    before_digest => {
      crate::engine::latex_constructs::before_float("table", None);
    },
    after_digest => sub[whatsit] {
      whatsit.set_property("float", floatflt_float_direction(whatsit));
      crate::engine::latex_constructs::after_float(whatsit);
    });
  DefMacro!("\\fltitem[]{}",    "\\item {#2}");
  DefMacro!("\\fltditem[]{}{}",  "\\item[#2] {#3}");
  DefMacro!("\\initfloatingfigs", "");
  DefMacro!("\\dofigtest", "");
  DefMacro!("\\dotabtest", "");
  DefMacro!("\\tryfig",    "");
  DefMacro!("\\trytab",    "");
  DefMacro!("\\figinsert", "");
  DefMacro!("\\tabinsert", "");
  DefMacro!("\\dohang",    "");
  DefMacro!("\\dohangt",   "");
  DefRegister!("\\ffigcount" => Number(0));
  DefRegister!("\\ftabcount" => Number(0));
  DefRegister!("\\fftest" =>    Number(0));
  DefRegister!("\\hangcount" => Number(0));
  DefRegister!("\\nosuccesstryfig" => Number(0));
  DefRegister!("\\nosuccesstrytab" => Number(0));
  DefRegister!("\\figgutter" => Dimension::from_str("1pc")?);
  DefRegister!("\\tabgutter" => Dimension::from_str("1pc")?);
  DefRegister!("\\htdone" =>      Number(0));
  DefRegister!("\\pageht" =>      Dimension::new(0));
  DefRegister!("\\startpageht" => Dimension::new(0));
  DefRegister!("\\tabbredd" =>      Dimension::new(0));
  DefRegister!("\\floatfltwidth" => Dimension::new(0));
  DefRegister!("\\fltitemwidth" =>  Dimension::new(0));
  // Perl L73-75,95: box registers and output test hook
  RawTeX!("\\newbox\\figbox");
  RawTeX!("\\newbox\\tabbox");
  RawTeX!("\\newbox\\pagebox");
  DefRegister!("\\outputpretest" => Tokens::new(vec![]));
  RawTeX!("\\newif\\iftryingfig     \\tryingfigfalse");
  RawTeX!("\\newif\\iftryingtab     \\tryingtabfalse");
  RawTeX!("\\newif\\ifdoingfig      \\doingfigfalse");
  RawTeX!("\\newif\\ifdoingtab      \\doingtabfalse");
  RawTeX!("\\newif\\iffigprocessing \\figprocessingfalse");
  RawTeX!("\\newif\\iftabprocessing \\tabprocessingfalse");
  RawTeX!("\\newif\\ifpageafterfig  \\pageafterfigfalse");
  RawTeX!("\\newif\\ifpageaftertab  \\pageaftertabfalse");
  RawTeX!("\\newif\\ifoddpages");
  RawTeX!("\\newif\\ifoutput");
});
