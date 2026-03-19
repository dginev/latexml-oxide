use crate::prelude::*;

LoadDefinitions!({
  // Perl: lxRDFa.sty.ltxml — LaTeXML support for RDFa
  // Minimal binding: define key commands so they don't produce ERROR nodes.
  // Full RDFa attribute handling deferred to future work.

  // DefKeyVal for the RDFa keyval family
  DefKeyVal!("RDFa", "about", "Semiverbatim");
  DefKeyVal!("RDFa", "resource", "Semiverbatim");
  DefKeyVal!("RDFa", "typeof", "Semiverbatim");
  DefKeyVal!("RDFa", "property", "Semiverbatim");
  DefKeyVal!("RDFa", "rel", "Semiverbatim");
  DefKeyVal!("RDFa", "rev", "Semiverbatim");
  DefKeyVal!("RDFa", "content", "Semiverbatim");
  DefKeyVal!("RDFa", "datatype", "Semiverbatim");

  // \lxRDFaPrefix{prefix}{url}
  DefMacro!("\\lxRDFaPrefix{}{}", None);

  // \lxRDFa[xpath]{keyvals} — absorb RDFa attributes (no DOM manipulation yet)
  DefMacro!("\\lxRDFa OptionalSemiverbatim RequiredKeyVals:RDFa", None);

  // \lxRDFAnnotate{keyvals}{text} — wrap text with RDFa attributes
  DefConstructor!("\\lxRDFAnnotate RequiredKeyVals:RDFa {}",
    "<ltx:text>#2</ltx:text>",
    enter_horizontal => true
  );

  // \lxRDF — no-op in both preamble and body for now
  DefMacro!("\\lxRDF@preamble[] RequiredKeyVals:RDFa", None);
  DefMacro!("\\lxRDF@body[] RequiredKeyVals:RDFa", None);
  Let!("\\lxRDF", "\\lxRDF@preamble");
  // Switch to body form at \begin{document}
  state::push_value("@at@begin@document",
    Tokens!(T_CS!("\\let"), T_CS!("\\lxRDF"), T_CS!("\\lxRDF@body")));
});
