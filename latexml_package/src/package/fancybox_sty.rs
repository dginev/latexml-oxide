use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // These could be made to depend on \fboxsep, \fboxrule, \cornersize
  DefMacro!("\\cornersize OptionalMatch:* {}", None);

  // Perl fancybox.sty.ltxml L25-36: DefConstructor(... mode => 'internal_vertical').
  // The mode declaration was dropped in the Rust stub — add it back so the
  // constructors pair like the Perl originals when they appear in
  // paragraph-mode contexts.
  DefConstructor!("\\shadowbox MoveableBox",
    "<ltx:text cssstyle='border:1px solid black; box-shadow: 5px 5px 10px black;'>#1</ltx:text>",
    mode => "internal_vertical");
  DefConstructor!("\\doublebox MoveableBox",
    "<ltx:text cssstyle='border:3px double black;'>#1</ltx:text>",
    mode => "internal_vertical");
  DefConstructor!("\\ovalbox MoveableBox",
    "<ltx:text cssstyle='border:1px solid black;border-radius:5px;'>#1</ltx:text>",
    mode => "internal_vertical");
  DefConstructor!("\\Ovalbox MoveableBox",
    "<ltx:text cssstyle='border:2px solid black;border-radius:5px;'>#1</ltx:text>",
    mode => "internal_vertical");

  // {Sbox} environment: saves its body for later use by \TheSbox
  // Perl stores the body via afterDigestBody + AssignValue/LookupValue.
  // For now, we approximate: {Sbox} is a no-op environment and \TheSbox is empty.
  DefEnvironment!("{Sbox}", "");
  DefMacro!("\\TheSbox", None);
});
