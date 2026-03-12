use crate::prelude::*;

LoadDefinitions!({
  // Choose the current float style (plain, plaintop, boxed, ruled)
  DefMacro!("\\float@style", None, "plain");
  DefMacro!("\\floatstyle{}", "\\def\\float@style{#1}");
  // \restylefloat{style} — ignore
  DefMacro!("\\restylefloat OptionalMatch:* {}", "");
  // \floatplacement{style}{placement} — ignore
  DefMacro!("\\floatplacement{}{}", "");
  // \listof{type}{title} — ignore
  DefMacro!("\\listof{}{}", "");
  // \floatname{type}{name}
  DefMacro!("\\floatname{}{}", "\\@namedef{lx@name@#1}{#2}");

  // \newfloat — simplified stub that creates basic environment
  // Full Perl creates environments with beforeFloat/afterFloat/addFloatFrames
  // For now, just define the counter and ignore the environment
  DefMacro!("\\newfloat{}{}{}[]", "");
});
