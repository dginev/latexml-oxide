use crate::prelude::*;
//======================================================================
// C.7.2 Common Structures
//======================================================================
// sub, superscript and prime are in TeX.pool
// Underlying support in TeX.pool.ltxml
LoadDefinitions!({
  // \stackrel{over}{base}: places "over" as a superscript over "base" relation
  DefMacro!("\\stackrel{}{}", r"\lx@stackrel{{\scriptstyle #1}}{{#2}}");
  DefConstructor!("\\lx@stackrel{}{}",
    "<ltx:XMApp role='RELOP'>\
      <ltx:XMTok role='SUPERSCRIPTOP' scriptpos='#scriptpos'/>\
      <ltx:XMArg>#2</ltx:XMArg>\
      <ltx:XMArg>#1</ltx:XMArg>\
    </ltx:XMApp>",
    reversion => "\\stackrel{#1}{#2}",
    properties => { stored_map!("scriptpos" => "mid") }
  );

  DefConstructor!(
    "\\frac InFractionStyle InFractionStyle",
    "<ltx:XMApp>\
      <ltx:XMTok meaning='divide' role='FRACOP' mathstyle='#mathstyle'/>\
      <ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg>\
      </ltx:XMApp>",
    properties => {
      let ms = lookup_font()
        .and_then(|f| f.get_mathstyle().map(|s| s.to_string()));
      match ms {
        Some(s) => Ok(stored_map!("mathstyle" => s)),
        None => Ok(stored_map!()),
      }
    }
  );
});
