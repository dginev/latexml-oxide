//! TeX Book, Appendix B. p. 361
use crate::package::*;

LoadDefinitions!({
  // This is actually LaTeX's definition, but let's just do it this way.
  DefConstructor!(
    "\\sqrt OptionalInScriptStyle Digested",
    "?#1(<ltx:XMApp><ltx:XMTok meaning='nth-root'/>\
    <ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>)\
    (<ltx:XMApp><ltx:XMTok role='FUNCTION' meaning='square-root'/><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>)"
  );


  DefParameterType!(ScriptStyleUntil, sub[_inner,until] {
    gullet_mut!().read_until(&until[0]) },
  before_digest => {
    stomach_mut!().bgroup();
    MergeFont!(mathstyle => "script");
  },
  after_digest => {
    stomach_mut!().egroup()?; },
  reversion => sub[args,_inner,_extra] {
      Ok(Tokens!(T_BEGIN!(), Tokens::new(args).revert(), T_END!())) });

  DefConstructor!("\\root ScriptStyleUntil:\\of {}",
    "<ltx:XMApp><ltx:XMTok meaning='nth-root'/>\
      <ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg>\
      </ltx:XMApp>",
    reversion => "\\root #1 \\of {#2}");

  //----------------------------------------------------------------------
  // LaTeX; Table 3.9. Log-like Functions, p.44.
  //----------------------------------------------------------------------
  // NOTE: Classifying some as TRIGFUNCTION might clarify 'pi' ambiguities ?
  DefMath!("\\arccos", None, "arccos", role => "OPFUNCTION", meaning => "inverse-cosine");
  DefMath!("\\arcsin", None, "arcsin", role => "OPFUNCTION", meaning => "inverse-sine");
  DefMath!("\\arctan", None, "arctan", role => "OPFUNCTION", meaning => "inverse-tangent");
  DefMath!("\\arg",    None, "arg",    role => "OPFUNCTION", meaning => "argument");

  DefMath!("\\cos",  None, "cos",  role => "TRIGFUNCTION", meaning => "cosine");
  DefMath!("\\cosh", None, "cosh", role => "TRIGFUNCTION", meaning => "hyperbolic-cosine");
  DefMath!("\\cot",  None, "cot",  role => "TRIGFUNCTION", meaning => "cotangent");
  DefMath!("\\coth", None, "coth", role => "TRIGFUNCTION", meaning => "hyperbolic-cotangent");

  DefMath!("\\csc", None, "csc", role => "TRIGFUNCTION", meaning => "cosecant");
  DefMath!("\\deg", None, "deg", role => "OPFUNCTION",   meaning => "degree");
  DefMath!("\\det", None, "det", role => "LIMITOP", meaning => "determinant",

    );//TODO: scriptpos => \&doScriptpos);
  DefMath!("\\dim", None, "dim", role => "LIMITOP", meaning => "dimension");

  DefMath!("\\exp", None, "exp", role => "OPFUNCTION", meaning => "exponential");
  DefMath!("\\gcd", None, "gcd", role => "OPFUNCTION", meaning => "gcd",

    );//TODO: scriptpos => \&doScriptpos);
  DefMath!("\\hom", None, "hom", role => "OPFUNCTION");
  DefMath!("\\inf", None, "inf", role => "LIMITOP", meaning => "infimum",

    );//TODO: scriptpos => \&doScriptpos);

  DefMath!("\\ker", None, "ker", role => "OPFUNCTION", meaning => "kernel");
  DefMath!("\\lg", None, "lg", role => "OPFUNCTION");
  DefMath!("\\lim", None, "lim", role => "LIMITOP", meaning => "limit",

    );//TODO: scriptpos => \&doScriptpos);
  DefMath!("\\liminf", None, "lim inf", role => "LIMITOP", meaning => "limit-infimum",

    );//TODO: scriptpos => \&doScriptpos);

  DefMath!("\\limsup", None, "lim sup", role => "LIMITOP", meaning => "limit-supremum",

    );//TODO: scriptpos => \&doScriptpos);
  DefMath!("\\ln",  None, "ln",  role => "OPFUNCTION", meaning => "natural-logarithm");
  DefMath!("\\log", None, "log", role => "OPFUNCTION", meaning => "logarithm");
  DefMath!("\\max", None, "max", role => "OPFUNCTION", meaning => "maximum",

    );//TODO: scriptpos => \&doScriptpos);

  DefMath!("\\min", None, "min", role => "OPFUNCTION", meaning => "minimum",

    );//TODO: scriptpos => \&doScriptpos);
  DefMath!("\\Pr",  None, "Pr",  role => "OPFUNCTION",
  );//TODO: scriptpos => \&doScriptpos);
  DefMath!("\\sec", None, "sec", role => "TRIGFUNCTION", meaning   => "secant");
  DefMath!("\\sin", None, "sin", role => "TRIGFUNCTION", meaning   => "sine");

  DefMath!("\\sinh", None, "sinh", role => "TRIGFUNCTION", meaning => "hyperbolic-sine");
  DefMath!("\\sup", None, "sup", role => "LIMITOP", meaning => "supremum",

    );//TODO: scriptpos => \&doScriptpos);
  DefMath!("\\tan",  None, "tan",  role => "TRIGFUNCTION", meaning => "tangent");
  DefMath!("\\tanh", None, "tanh", role => "TRIGFUNCTION", meaning => "hyperbolic-tangent");

  //----------------------------------------------------------------------
  // Modulo

  DefMath!("\\pmod{}", r"\;\;(\mathop{{\rm mod}} #1)", role => "MODIFIER");  //  , meaning=>"modulo");
  DefMath!("\\bmod", "mod", role => "MODIFIEROP", meaning => "modulo");


});
