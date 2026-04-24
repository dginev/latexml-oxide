use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("numprint", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Override numprint's `n` and `N` column type rewrites. The raw package
  // defines these using \nprt@rewrite@ with \@ifnextchar and \nprt@digittoks
  // which produce unrecognized tokens in our alignment template parser.
  // Simplify to plain right-aligned columns (loses decimal alignment but
  // prevents 54+ stray alignment errors).
  RawTeX!(r#"\makeatletter
\renewcommand{\NC@rewrite@n}[1]{\NC@find r}%
\renewcommand{\NC@rewrite@N}[1]{\NC@find r}%
\makeatother"#);

  Let!("\\ltx@orig@numprint", "\\numprint");
  DefMacro!("\\numprint[]{}",
    "\\ifx.#1.\\ltx@numprint@{#2}\\else\\ltx@numprint@@{#1}{#2}\\fi");
  DefMacro!("\\ltx@numprint@{}",
    "\\ifmmode\\ltx@math@numprint@{#1}\\else\\ltx@text@numprint@{#1}\\fi");
  DefMacro!("\\ltx@numprint@@{}{}",
    "\\ifmmode\\ltx@math@numprint@@{#1}{#2}\\else\\ltx@text@numprint@@{#1}{#2}\\fi");
  DefMacro!("\\ltx@text@numprint@{}",    "\\ltx@text@number{\\ltx@orig@numprint{#1}}");
  DefMacro!("\\ltx@text@numprint@@{}{}", "\\ltx@text@number{\\ltx@orig@numprint[#1]{#2}}");
  // \ltx@text@number: Perl numprint.sty.ltxml L48-50 defines this as
  //   DefConstructor('\ltx@text@number{}',
  //     "<ltx:text class='ltx_number' _noautoclose='1'>#1</ltx:text>",
  //     enterHorizontal => 1);
  // — a single DefConstructor. Rust splits into a DefMacro trampoline
  // that `\ifmmode`-guards the wrap, then delegates to a hidden
  // `\ltx@text@number@wrap` DefConstructor. Without the math-mode
  // guard, `ltx:text` emits inside `<ltx:XMath>` which the validator
  // rejects; Perl avoids this by its `\numprint` caller never being
  // reachable from math mode in the same way as the Rust port.
  // Intentional DefConstructor → DefMacro kind divergence; see
  // WISDOM #44 + OXIDIZED_DESIGN for the ltx:text-in-math invariant.
  DefMacro!("\\ltx@text@number{}",
    "\\ifmmode#1\\else\\ltx@text@number@wrap{#1}\\fi");
  // Perl numprint.sty.ltxml has `enterHorizontal => 1` on
  // \ltx@text@number — Rust ifmmode-guards through to
  // \ltx@text@number@wrap, so the flag belongs on the wrap
  // constructor. Without it, a `\numprint{42}` between paragraphs
  // (text mode, vertical context) emits the number-class <ltx:text>
  // as a stray block-level child.
  DefConstructor!("\\ltx@text@number@wrap{}",
    "<ltx:text class='ltx_number' _noautoclose='1'>#1</ltx:text>",
    enter_horizontal => true);
  DefMacro!("\\ltx@math@numprint@{}",
    "\\ltx@math@@numprint@{#1}{\\ltx@orig@numprint{#1}}");
  DefMacro!("\\ltx@math@numprint@@{}{}",
    "\\ltx@math@@numprint@@{#1}{#2}{\\ltx@mark@units{#1}}{\\ltx@orig@numprint[#1]{#2}}");

  // Math constructors (Perl L59-76). Perl sets
  //   reversion => '\numprint{#1}' / '\numprint[#1]{#2}'
  // so that the internal CS round-trips to the user-facing `\numprint`
  // form in `tex=` attributes. Without them, reversion would emit the
  // private `\ltx@math@@numprint@` name — breaking any consumer that
  // reconstructs LaTeX source from the XML (UnTeX, math export).
  DefConstructor!("\\ltx@math@@numprint@ {} {}",
    "<ltx:XMDual>\
       <ltx:XMTok meaning='#value' role='NUMBER'>#value</ltx:XMTok>\
       <ltx:XMWrap>#2</ltx:XMWrap>\
     </ltx:XMDual>",
    reversion => "\\numprint{#1}",
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
    reversion => "\\numprint[#1]{#2}",
    properties => sub[args] {
      let value = args.get(1).and_then(|a| a.as_ref())
        .map(|a| a.to_string()).unwrap_or_default();
      Ok(stored_map!("value" => value))
    });

  // Unit marking (Perl L99-111): simplified — just absorb content
  DefConstructor!("\\ltx@mark@units{}", "#1", reversion => "#1");

  // Sign symbols (Perl L79-84)
  DefPrimitive!("\\ltx@text@plus", "+");
  DefPrimitive!("\\ltx@text@minus", "-");
  DefPrimitive!("\\ltx@text@plusminus", "\u{00B1}");
  DefMacro!(T_CS!("\\nprt@sign@+"),  None, "\\ifmmode+\\else\\ltx@text@plus\\fi");
  DefMacro!(T_CS!("\\nprt@sign@-"),  None, "\\ifmmode-\\else\\ltx@text@minus\\fi");
  DefMacro!(T_CS!("\\nprt@sign@+-"), None, "\\ifmmode\\pm\\else\\ltx@text@plusminus\\fi");

  // Product sign (Perl L87-94)
  // CS names with special chars — use RawTeX to define
  RawTeX!(r"\expandafter\def\csname ltx@text@prod\string\times\endcsname{×}");
  RawTeX!(r"\expandafter\def\csname ltx@text@prod\string\cdot\endcsname{⋅}");

  // Product sign: override \nprt@prod to use text × directly (Perl L87-94)
  DefMacro!("\\npproductsign{}",
    "\\ifmmode #1\\else\\@ifundefined{ltx@text@prod\\string #1}{\\def\\nprt@prod{\\ensuremath{{}#1{}}}}{\\def\\nprt@prod{\\csname ltx@text@prod\\string #1\\endcsname}}\\fi");
  // Directly set \nprt@prod to text × (the raw package sets it to \ensuremath{}\times{})
  RawTeX!(r"\makeatletter\def\nprt@prod{×}\makeatother");

  DefMacro!("\\npunitcommand{}", "\\ensuremath{\\mathrm{\\ltx@mark@units #1}}");
});
