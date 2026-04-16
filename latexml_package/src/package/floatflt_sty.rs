use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  state::assign_value("floatfltpos", Stored::from("v"), None);
  DeclareOption!("rflt", { state::assign_value("floatfltpos", Stored::from("r"), None); });
  DeclareOption!("lflt", { state::assign_value("floatfltpos", Stored::from("l"), None); });
  DeclareOption!("pflt", { state::assign_value("floatfltpos", Stored::from("p"), None); });
  DeclareOption!("vflt", { state::assign_value("floatfltpos", Stored::from("v"), None); });
  DefEnvironment!("{floatingfigure}[]{Dimension}",
    "<ltx:figure xml:id='#id' inlist='#inlist' float='right'>#tags #body</ltx:figure>",
    before_digest => {
      crate::engine::latex_constructs::before_float("figure", None);
    },
    after_digest_body => sub[whatsit] {
      crate::engine::latex_constructs::after_float(whatsit);
    });
  DefEnvironment!("{floatingtable}[]{Dimension}",
    "<ltx:table xml:id='#id' inlist='#inlist' float='right'>#tags #body</ltx:table>",
    before_digest => {
      crate::engine::latex_constructs::before_float("table", None);
    },
    after_digest_body => sub[whatsit] {
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
