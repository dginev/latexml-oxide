use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\subdef{}", "");
  DefMacro!("\\Color{}{}", "{\\textColor{#1} #2}");
  InputDefinitions!("dvipsnam", extension => Some(Cow::Borrowed("def")));
});
