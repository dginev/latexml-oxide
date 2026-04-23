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
  // In text mode, \numprint wraps output in ltx:text class="ltx_number".
  // Port of Perl: DefConstructor('\ltx@text@number{}',
  //   "<ltx:text class='ltx_number' _noautoclose='1'>#1</ltx:text>",
  //   enterHorizontal => 1);
  // Use \ifmmode guard: skip wrapping in math mode (ltx:text invalid inside XMath).
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
