//! TeX Book, Appendix B. p. 361
use crate::package::*;

LoadDefinitions!(state, {

// This is actually LaTeX's definition, but let's just do it this way.
DefConstructor!("\\sqrt OptionalInScriptStyle Digested",
  "?#1(<ltx:XMApp><ltx:XMTok meaning='nth-root'/>\
    <ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg>\
    </ltx:XMApp>)\
    (<ltx:XMApp><ltx:XMTok meaning='square-root'/>\
    <ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>)");

});
