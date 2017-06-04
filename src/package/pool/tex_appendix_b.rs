use package::*;
use rtx_core::{BoxOps};
pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);
//
//**********************************************************************
// Plain;  Extracted from Appendix B.
//**********************************************************************
//
//======================================================================
// TeX Book, Appendix B, p. 344
//======================================================================
// \dospecials ??
//
// Normally, the content branch contains the pure structure and meaning of a construct,
// and the presentation is generated from lower level TeX macros that only concern
// themselves with how to display the object.
// Nevertheless, it is sometimes useful to know where the tokens in the presentation branch
// came from;  particularly what their presumed "meaning" is.
// For example, when search-indexing pmml, or providing links to definitions from the pmml.
//
// The following constructor (see how it's used in DefMath), adds meaning attributes
// whereever it seems sensible on the presentation branch, after it has been generated.

// DefConstructor('\@ASSERT@MEANING{}{}', '#2',
//   reversion      => '#2',
//   afterConstruct => sub {
//     my ($document, $whatsit) = @_;
//     my $node    = $document->getNode;              # This should be the wrapper just added.
//     my $meaning = ToString($whatsit->getArg(1));
//     addMeaningRec($document, $node, $meaning);
//     $node; });

//======================================================================
// Properties for plain characters.
// These are allowed in plain text, but need to act a bit special in math.
  DefMathI!('=', None, '=', role => v!("RELOP"),   meaning  => v!("equals"));
  DefMathI!('+', None, '+', role => v!("ADDOP"),   meaning  => v!("plus"));
  DefMathI!('-', None, '-', role => v!("ADDOP"),   meaning  => v!("minus"));
  DefMathI!('*', None, '*', role => v!("MULOP"),   meaning  => v!("times"));
  DefMathI!('/', None, '/', role => v!("MULOP"),   meaning  => v!("divide"));
  DefMathI!('!', None, '!', role => v!("POSTFIX"), meaning  => v!("factorial"));
  DefMathI!(',', None, ',', role => v!("PUNCT"));
  DefMathI!('.', None, '.', role => v!("PERIOD"));
  DefMathI!(';', None, ';', role => v!("PUNCT"));
  DefMathI!('(', None, '(', role => v!("OPEN"),    stretchy => false);
  DefMathI!(')', None, ')', role => v!("CLOSE"),   stretchy => false);
  DefMathI!('[', None, '[', role => v!("OPEN"),    stretchy => false);
  DefMathI!(']', None, ']', role => v!("CLOSE"),   stretchy => false);
  DefMathI!('|', None, '|', role => v!("VERTBAR"), stretchy => false);
  DefMathI!(':', None, ':', role => v!("METARELOP"), name => v!("colon"));    // Seems like good default role
  DefMathI!('<', None, '<', role => v!("RELOP"), meaning => v!("less-than"));
  DefMathI!('>', None, '>', role => v!("RELOP"), meaning => v!("greater-than"));

  Ok(())
}
