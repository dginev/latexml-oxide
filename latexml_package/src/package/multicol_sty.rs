use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: multicol.sty.ltxml
  // Since this is basically styling, we can ignore the effects.

  // Optional arg is sortof a heading, but w/o any particular styling(?)
  // Note: Perl uses <ltx:para> (open, not close) at end of conditional
  // to auto-close the heading para and open a new one that wraps pagination + body
  // Body is digested in `internal_vertical` so `$$ … $$` display-math
  // gate (`tex_math.rs:443` / Perl `TeX_Math.pool.ltxml:65`) recognizes
  // the second `$`. Without this, multicols's body inherits/defaults to
  // restricted_horizontal and `$$ x_1 $$` errors with `Script _ can only
  // appear in math mode`. Faithful Perl-template wrapping is preserved
  // (Perl multicol.sty.ltxml:21-25) — only the digest mode is made
  // explicit. Cluster: cond-mat0001099 + hep-ph0001306 + math0601451's
  // many script-mode errors all trace here.
  DefEnvironment!("{multicols}{}[]",
    r###"?#2(<ltx:para><ltx:p>#2</ltx:p><ltx:para>)<ltx:pagination role='start_#1_columns'/>#body<ltx:pagination role='end_#1_columns'/>"###,
    mode => "internal_vertical");
  DefEnvironment!("{multicols*}{}[]",
    r###"?#2(<ltx:para><ltx:p>#2</ltx:p><ltx:para>)<ltx:pagination role='start_#1_columns'/>#body<ltx:pagination role='end_#1_columns'/>"###,
    mode => "internal_vertical");

  def_macro_noop("\\botmark")?;
  def_macro_noop("\\topmark")?;

  def_macro_noop("\\flushcolumns")?;
  def_macro_noop("\\raggedcolumns")?;
  def_macro_noop("\\setemergencystretch")?;

  DefRegister!("\\premulticols"         => Dimension!("50pt"));
  DefRegister!("\\postmulticols"        => Dimension!("20pt"));
  DefRegister!("\\multicolsep"          => Glue!("12pt plus 4pt minus 3pt"));
  DefRegister!("\\multicolbaselineskip" => Glue::new(0));
  DefRegister!("\\multicolovershoot"    => Dimension!("2pt"));
  DefRegister!("\\multicolundershoot"   => Dimension!("2pt"));
  DefRegister!("\\multicolpretolerance" => Number::new(-1));
  DefRegister!("\\multicoltolerance"    => Number::new(9999));
  DefRegister!("\\doublecol@number"     => Number::new(0));
});
