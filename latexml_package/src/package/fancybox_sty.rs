use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // These could be made to depend on \fboxsep, \fboxrule, \cornersize
  DefMacro!("\\cornersize OptionalMatch:* {}", None);

  DefConstructor!("\\shadowbox MoveableBox",
    "<ltx:text cssstyle='border:1px solid black; box-shadow: 5px 5px 10px black;'>#1</ltx:text>");
  DefConstructor!("\\doublebox MoveableBox",
    "<ltx:text cssstyle='border:3px double black;'>#1</ltx:text>");
  DefConstructor!("\\ovalbox MoveableBox",
    "<ltx:text cssstyle='border:1px solid black;border-radius:5px;'>#1</ltx:text>");
  DefConstructor!("\\Ovalbox MoveableBox",
    "<ltx:text cssstyle='border:2px solid black;border-radius:5px;'>#1</ltx:text>");

  // {Sbox} environment: saves its body for later use by \TheSbox
  // Perl stores the body via afterDigestBody + AssignValue/LookupValue.
  // For now, we approximate: {Sbox} is a no-op environment and \TheSbox is empty.
  DefEnvironment!("{Sbox}", "");
  DefMacro!("\\TheSbox", None);
});
