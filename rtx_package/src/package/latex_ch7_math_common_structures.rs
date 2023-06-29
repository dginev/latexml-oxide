use crate::package::*;
//======================================================================
// C.7.2 Common Structures
//======================================================================
// sub, superscript and prime are in TeX.pool
// Underlying support in TeX.pool.ltxml
LoadDefinitions!({
  DefConstructor!("\\frac InFractionStyle InFractionStyle",
    "<ltx:XMApp>\
      <ltx:XMTok meaning='divide' role='FRACOP' mathstyle='#mathstyle'/>\
      <ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg>\
      </ltx:XMApp>"
      // TODO
    // sizer      => sub[whatsit] { frac_sizer(whatsit.get_arg(1), whatsit.get_arg(2)) },
    // properties => { "mathstyle" => lookup_font().get_mathstyle() }
  );
});
