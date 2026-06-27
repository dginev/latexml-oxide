use crate::prelude::*;

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
  } else {
    match lookup_value("floatfltpos") {
      Some(Stored::String(sym)) => with(sym, |s| s.to_string()),
      _ => "v".to_string(),
    }
  };
  if pos.starts_with('v') || pos.starts_with('r') {
    "right"
  } else {
    "left"
  }
}

/// Perl `toPercent($_[2])` (floatfig.sty.ltxml L41-43):
/// `int(100 * $dimen->valueOf / LookupValue('\textwidth')->valueOf) . '%'`.
/// `lookup_dimension` mirrors Perl's `LookupValue('\textwidth')` (value table,
/// never warns about register-ness). Must run where the env args exist
/// (after_digest_begin). See floatflt_sty.rs for the full rationale.
fn floatfig_pct_width(whatsit: &Whatsit) -> String {
  let Some(dim_arg) = whatsit.get_arg(2) else {
    return String::new();
  };
  let Some(tw) = lookup_dimension("\\textwidth") else {
    return String::new();
  };
  let tw_sp = tw.value_of();
  if tw_sp == 0 {
    return String::new();
  }
  let pct = (100 * dim_arg.value_of()) / tw_sp;
  s!("{pct}%")
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: floatfig.sty.ltxml — floating figures (restricted floatflt)

  assign_value("floatfltpos", Stored::from("v"), None);
  DeclareOption!("rflt", { assign_value("floatfltpos", Stored::from("r"), None); });
  DeclareOption!("lflt", { assign_value("floatfltpos", Stored::from("l"), None); });
  DeclareOption!("pflt", { assign_value("floatfltpos", Stored::from("p"), None); });
  DeclareOption!("vflt", { assign_value("floatfltpos", Stored::from("v"), None); });

  // Perl: DefEnvironment('{floatingfigure}[]{Dimension}', ...)
  // Simplified: just wrap in ltx:figure.
  // NOTE: uses `after_digest` (not `after_digest_body`) to match engine's
  // {figure} / {table} — after_digest_body runs after frame pop, by which
  // time `\@captype` (set locally by before_float) is gone, and
  // after_float's `digest(\@captype)` would error. See floatflt_sty.rs
  // commit 2b57844c4 for details.
  // float/pctwidth need the env args, which live on the BEGIN whatsit
  // (after_digest_begin); the END whatsit in after_digest has none (get_arg →
  // None → `width="0%"`). Compute in after_digest_begin, keep after_float in
  // after_digest (needs live `\@captype`). See floatflt_sty.rs for full details.
  DefEnvironment!("{floatingfigure}[]{Dimension}",
    "<ltx:figure xml:id='#id' inlist='#inlist' float='#float' width='#pctwidth'>#tags #body</ltx:figure>",
    before_digest => {
      engine::latex_constructs::before_float("figure", None);
    },
    after_digest_begin => sub[whatsit] {
      whatsit.set_property("float", floatfig_float_direction(whatsit));
      whatsit.set_property("pctwidth", Stored::from(floatfig_pct_width(whatsit)));
    },
    after_digest => sub[whatsit] {
      engine::latex_constructs::after_float(whatsit);
    });

  DefMacro!("\\fltitem[]{}", "\\item {#2}");
  DefMacro!("\\fltditem[]{}{}", "\\item[#2] {#3}");

  def_macro_noop("\\initfloatingfigs")?;
  def_macro_noop("\\dofigtest")?;
  def_macro_noop("\\tryfig")?;
  def_macro_noop("\\figinsert")?;
  def_macro_noop("\\dohang")?;

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
