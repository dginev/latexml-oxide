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

  // Perl fancybox.sty.ltxml L38-46: {Sbox} stashes its digested body
  // globally under the `Sbox` state value; \TheSbox pops it and
  // replays the stored content in place. Prior Rust stub was an empty
  // env + no-op macro, so `\sbox{…}{foo}\TheSbox` lost `foo`.
  DefEnvironment!("{Sbox}", "",
    after_digest_body => sub[whatsit] {
      if let Ok(Some(body)) = whatsit.get_body() {
        assign_value("Sbox", Stored::Digested(body), Some(Scope::Global));
      }
    });
  DefPrimitive!("\\TheSbox", {
    let stashed = lookup_value("Sbox");
    assign_value("Sbox", Stored::None, Some(Scope::Global));
    if let Some(Stored::Digested(body)) = stashed {
      return Ok(vec![body]);
    }
  });
});
