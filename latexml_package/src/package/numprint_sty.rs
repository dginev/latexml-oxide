use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("numprint", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  Let!("\\ltx@orig@numprint", "\\numprint");
  DefMacro!("\\numprint[]{}",
    "\\ifx.#1.\\ltx@numprint@{#2}\\else\\ltx@numprint@@{#1}{#2}\\fi");
  DefMacro!("\\ltx@numprint@{}",
    "\\ifmmode\\ltx@math@numprint@{#1}\\else\\ltx@text@numprint@{#1}\\fi");
  DefMacro!("\\ltx@numprint@@{}{}",
    "\\ifmmode\\ltx@math@numprint@@{#1}{#2}\\else\\ltx@text@numprint@@{#1}{#2}\\fi");
  DefMacro!("\\ltx@text@numprint@{}",    "\\ltx@text@number{\\ltx@orig@numprint{#1}}");
  DefMacro!("\\ltx@text@numprint@@{}{}", "\\ltx@text@number{\\ltx@orig@numprint[#1]{#2}}");
  DefConstructor!("\\ltx@text@number{}",
    "<ltx:text class='ltx_number' _noautoclose='1'>#1</ltx:text>");
  DefMacro!("\\ltx@math@numprint@{}",
    "\\ltx@math@@numprint@{#1}{\\ltx@orig@numprint{#1}}");
  DefMacro!("\\nprt@sign@+",  "\\ifmmode+\\else+\\fi");
  DefMacro!("\\nprt@sign@-",  "\\ifmmode-\\else-\\fi");
  DefMacro!("\\nprt@sign@+-", "\\ifmmode\\pm\\else\\pm\\fi");
  DefMacro!("\\npunitcommand{}", "\\ensuremath{\\mathrm{#1}}");
});
