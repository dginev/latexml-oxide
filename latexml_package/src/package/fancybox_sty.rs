use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Load the real fancybox.sty first (pure LaTeX/TeX box code: \shadowsize,
  // \VerbBox, the B-environments {Bcenter}…{Beqnarray*}, \boxput, \fancyoval,
  // \fancypage/\fancyput/\Landscape, and its verbatim layer \Verb/{Verbatim}/
  // \SaveVerb/\UseVerb…). Perl's fancybox.sty.ltxml covers only the four
  // frame boxes + {Sbox}; the TL doc corpus witness fancybox/fancybox-doc
  // exercises the whole API in {example} demos (which write \jobname.tmp
  // through {VerbatimOut} and re-input it), so every unbound command was an
  // `undefined` error once those demos actually executed. The Perl-shaped
  // overrides below are applied AFTER the raw load, so they win.
  InputDefinitions!("fancybox", noltxml => true, extension => Some(Cow::Borrowed("sty")));

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

  // OXIDIZED_DESIGN #192: the raw package's swap cannot displace the locked
  // LaTeXML footnote constructors. Route the package-scoped command to the
  // kernel's live-body implementation.
  DefMacro!("\\VerbatimFootnotes", "\\lx@VerbatimFootnotes");
});
