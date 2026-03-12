use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: multicol.sty.ltxml
  // Since this is basically styling, we can ignore the effects.

  // Optional arg is sortof a heading, but w/o any particular styling(?)
  DefEnvironment!("{multicols}{}[]",
    r###"?#2(<ltx:para><ltx:p>#2</ltx:p></ltx:para>)<ltx:pagination role='start_#1_columns'/>#body<ltx:pagination role='end_#1_columns'/>"###);
  DefEnvironment!("{multicols*}{}[]",
    r###"?#2(<ltx:para><ltx:p>#2</ltx:p></ltx:para>)<ltx:pagination role='start_#1_columns'/>#body<ltx:pagination role='end_#1_columns'/>"###);

  DefMacro!("\\botmark", "");
  DefMacro!("\\topmark", "");

  DefMacro!("\\flushcolumns", "");
  DefMacro!("\\raggedcolumns", "");
  DefMacro!("\\setemergencystretch", "");

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
