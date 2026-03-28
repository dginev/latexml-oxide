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
  DefMacro!("\\ltx@math@numprint@@{}{}",
    "\\ltx@math@@numprint@@{#1}{#2}{\\ltx@mark@units{#1}}{\\ltx@orig@numprint[#1]{#2}}");

  // Math constructors (Perl L59-76)
  DefConstructor!("\\ltx@math@@numprint@ {} {}",
    "<ltx:XMDual>\
       <ltx:XMTok meaning='#value' role='NUMBER'>#value</ltx:XMTok>\
       <ltx:XMWrap>#2</ltx:XMWrap>\
     </ltx:XMDual>",
    properties => sub[args] {
      let value = args.first().and_then(|a| a.as_ref())
        .map(|a| a.to_string()).unwrap_or_default();
      Ok(stored_map!("value" => value))
    });
  DefConstructor!("\\ltx@math@@numprint@@ {} {} {} {}",
    "<ltx:XMDual>\
       <ltx:XMApp>\
         <ltx:XMTok meaning='times' role='MULOP'>\u{2062}</ltx:XMTok>\
         <ltx:XMTok meaning='#value' role='NUMBER'>#value</ltx:XMTok>\
         <ltx:XMWrap>#3</ltx:XMWrap>\
       </ltx:XMApp>\
       <ltx:XMWrap>#4</ltx:XMWrap>\
     </ltx:XMDual>",
    properties => sub[args] {
      let value = args.get(1).and_then(|a| a.as_ref())
        .map(|a| a.to_string()).unwrap_or_default();
      Ok(stored_map!("value" => value))
    });

  // Unit marking (Perl L99-111): simplified — just absorb content
  DefConstructor!("\\ltx@mark@units{}", "#1", reversion => "#1");

  // Sign symbols (Perl L79-84)
  DefMacro!(T_CS!("\\nprt@sign@+"),  None, "\\ifmmode+\\else+\\fi");
  DefMacro!(T_CS!("\\nprt@sign@-"),  None, "\\ifmmode-\\else-\\fi");
  DefMacro!(T_CS!("\\nprt@sign@+-"), None, "\\ifmmode\\pm\\else\\pm\\fi");

  // Product sign (Perl L87-94)
  // CS names with special chars — use RawTeX to define
  RawTeX!(r"\expandafter\def\csname ltx@text@prod\string\times\endcsname{×}");
  RawTeX!(r"\expandafter\def\csname ltx@text@prod\string\cdot\endcsname{⋅}");

  DefMacro!("\\npunitcommand{}", "\\ensuremath{\\mathrm{#1}}");
});
