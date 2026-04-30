use crate::prelude::*;
use latexml_core::definition::register::RegisterValue;

/// Perl floatfig.sty.ltxml L37-39: same direction-from-arg logic as
/// floatflt — `v`/`r` prefix → right, else left.
fn floatfig_float_direction(whatsit: &Whatsit) -> &'static str {
  let pos_arg = whatsit
    .get_arg(1)
    .map(|a| a.to_string())
    .unwrap_or_default();
  let pos_arg = pos_arg.trim().to_string();
  let pos = if !pos_arg.is_empty() {
    pos_arg
  } else if let Some(Stored::String(sym)) = state::lookup_value("floatfltpos") {
    arena::with(sym, |s| s.to_string())
  } else {
    "v".to_string()
  };
  if pos.starts_with('v') || pos.starts_with('r') {
    "right"
  } else {
    "left"
  }
}

/// Perl `toPercent($_[2])` — 100 * dimen / \textwidth, formatted as "NN%".
fn floatfig_pct_width(whatsit: &Whatsit) -> String {
  let dim_arg = whatsit
    .get_arg(2)
    .map(|a| a.to_attribute())
    .unwrap_or_default();
  let Ok(dim) = Dimension::from_str(&dim_arg) else {
    return String::new();
  };
  let tw = match state::lookup_register("\\textwidth", Vec::new()) {
    Ok(Some(RegisterValue::Dimension(d))) => d.value_of(),
    _ => return String::new(),
  };
  if tw == 0 {
    return String::new();
  }
  let pct = (100 * dim.value_of()) / tw;
  s!("{pct}%")
}

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
    "<ltx:figure xml:id='#id' inlist='#inlist' float='#float' width='#pctwidth'>#tags #body</ltx:figure>",
    before_digest => {
      crate::engine::latex_constructs::before_float("figure", None);
    },
    after_digest => sub[whatsit] {
      whatsit.set_property("float", floatfig_float_direction(whatsit));
      whatsit.set_property("pctwidth", Stored::from(floatfig_pct_width(whatsit)));
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
  // Perl L64: figgutter default is 1pc, not 0pt.
  DefRegister!("\\figgutter", Dimension::from_str("1pc")?);
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
