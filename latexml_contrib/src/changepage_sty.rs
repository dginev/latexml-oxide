use latexml_package::prelude::*;


LoadDefinitions!({
  DeclareOption!("strict", "");
  DefConditional!("\\ifoddpage");
  DefConditional!("\\ifstrictpagecheck");
  DefConditional!("\\ifcpstrict");
  DefConditional!("\\ifcpoddpage");
  def_macro_noop("\\strictpagecheck")?;
  def_macro_noop("\\easypagecheck")?;
  def_macro_noop("\\pmemlabel{}")?;
  def_macro_noop("\\newpmemlabel{}{}")?;
  def_macro_noop("\\pmemlabelref{}")?;
  def_macro_noop("\\checkoddpage")?;
  def_macro_noop("\\cplabelprefix")?;
  def_macro_noop("\\cplabel{}")?;
  def_macro_noop("\\newcplabel{}{}")?;
  // Mode must be internal_vertical so `$$`-display-math inside the env
  // is recognised by the dollar handler (tex_math.rs's
  // `BOUND_MODE.ends_with("vertical")` gate). Without this, witness
  // 2305.09826: `$$\log_2(x)$$` inside `\begin{adjustwidth}{...}{...}`
  // emitted Error:unexpected:_ "Script _ can only appear in math mode"
  // for every `_` in the formula because the wrapping env's default
  // restricted_horizontal mode kept the `$` handler from entering
  // display math. See sibling fix in
  // latexml_package/src/package/changepage_sty.rs.
  DefEnvironment!("{adjustwidth} OptionalMatch:* []{}{}", "#body",
    mode => "internal_vertical");
  // `adjustwidth*` is a SEPARATE environment in changepage.sty (L122
  // `\newenvironment{adjustwidth*}[2]`) — the `*` is part of the env NAME,
  // not a `*`-argument, so the `OptionalMatch:*` on `{adjustwidth}` above
  // never matches it. Perl has no changepage binding and raw-loads the real
  // .sty, which defines both; our stub previously defined only the unstarred
  // form, so `\begin{adjustwidth*}{..}{..}` (odd/even-page margin variant)
  // raised "The environment {adjustwidth*} is not defined" (witness
  // 2006.09676). Mirror the unstarred env (the odd/even-page distinction in
  // the real def is moot for our paradigm — both branches just set list
  // margins, which we ignore). Same `internal_vertical` mode fix for
  // `$$`-display-math inside the env.
  DefEnvironment!("{adjustwidth*} []{}{}", "#body",
    mode => "internal_vertical");
  def_macro_noop("\\changetext{}{}{}{}{}")?;
  def_macro_noop("\\changepage{}{}{}{}{}{}{}{}{}")?;
});
