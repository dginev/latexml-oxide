use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: floatfig.sty.ltxml — floating figures (restricted floatflt)

  state::assign_value("floatfltpos", Stored::from("v"), None);
  DeclareOption!("rflt", { state::assign_value("floatfltpos", Stored::from("r"), None); });
  DeclareOption!("lflt", { state::assign_value("floatfltpos", Stored::from("l"), None); });
  DeclareOption!("pflt", { state::assign_value("floatfltpos", Stored::from("p"), None); });
  DeclareOption!("vflt", { state::assign_value("floatfltpos", Stored::from("v"), None); });

  // Perl: DefEnvironment('{floatingfigure}[]{Dimension}', ...)
  // Simplified: just wrap in ltx:figure.
  // NOTE: uses `after_digest` (not `after_digest_body`) to match engine's
  // {figure} / {table} — after_digest_body runs after frame pop, by which
  // time `\@captype` (set locally by before_float) is gone, and
  // after_float's `digest(\@captype)` would error. See floatflt_sty.rs
  // commit 2b57844c4 for details.
  DefEnvironment!("{floatingfigure}[]{Dimension}",
    "<ltx:figure xml:id='#id' inlist='#inlist' float='right'>#tags #body</ltx:figure>",
    before_digest => {
      crate::engine::latex_constructs::before_float("figure", None);
    },
    after_digest => sub[whatsit] {
      crate::engine::latex_constructs::after_float(whatsit);
    });

  DefMacro!("\\fltitem[]{}", "\\item {#2}");
  DefMacro!("\\fltditem[]{}{}", "\\item[#2] {#3}");

  DefMacro!("\\initfloatingfigs", None);
  DefMacro!("\\dofigtest", None);
  DefMacro!("\\tryfig",    None);
  DefMacro!("\\figinsert", None);
  DefMacro!("\\dohang",    None);

  DefRegister!("\\ffigcount", Number(0));
  DefRegister!("\\fftest",    Number(0));
  DefRegister!("\\hangcount", Number(0));

  DefRegister!("\\nosuccesstryfig", Number(0));
  DefRegister!("\\figgutter", Dimension(0));
  DefRegister!("\\htdone",      Number(0));
  DefRegister!("\\pageht",      Dimension(0));
  DefRegister!("\\startpageht", Dimension(0));
  DefRegister!("\\floatfltwidth", Dimension(0));
  DefRegister!("\\fltitemwidth",  Dimension(0));
  // Box registers + output hook (matches floatflt)
  RawTeX!("\\newbox\\figbox");
  RawTeX!("\\newbox\\pagebox");
  DefRegister!("\\outputpretest", Tokens::new(vec![]));

  RawTeX!("\\newif\\iftryingfig     \\tryingfigfalse");
  RawTeX!("\\newif\\ifdoingfig      \\doingfigfalse");
  RawTeX!("\\newif\\iffigprocessing \\figprocessingfalse");
  RawTeX!("\\newif\\ifpageafterfig  \\pageafterfigfalse");
  RawTeX!("\\newif\\ifoddpages");
  RawTeX!("\\newif\\ifoutput");
});
