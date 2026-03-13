use crate::prelude::*;

LoadDefinitions!({
  DefMath!("\\ring{}", "\u{030A}", operator_role => "OVERACCENT");

  DefMacro!("\\lx@acc@size", "\\scriptstyle");

  DefMacro!("\\accentset{}{}", "\\lx@overaccentset{#1}{#2}");
  DefConstructor!("\\lx@overaccentset ScriptStyle {}",
    "<ltx:XMApp><ltx:XMWrap role='OVERACCENT'>#1</ltx:XMWrap><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>",
    sizer => "#1",
    alias => "\\accentset");

  DefMath!("\\dddot{}", "\u{02D9}\u{02D9}\u{02D9}", operator_role => "OVERACCENT");
  DefMath!("\\ddddot{}", "\u{02D9}\u{02D9}\u{02D9}\u{02D9}", operator_role => "OVERACCENT");

  // \underaccent{acc}{base} — simplified version
  // The Perl version checks if the arg is an accent command; we simplify
  DefMacro!("\\underaccent{}{}", "\\lx@underaccentset{#1}{#2}");
  DefConstructor!("\\lx@underaccentset ScriptStyle {}",
    "<ltx:XMApp><ltx:XMWrap role='UNDERACCENT'>#1</ltx:XMWrap><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>",
    sizer => "#1",
    alias => "\\underaccent");

  DefMath!("\\undertilde{}", "\u{007E}", operator_role => "UNDERACCENT");
});
