use crate::prelude::*;

//**********************************************************************
// LaTeXML Specific.
// Support for Declarations & Presentation/Semantic Duality
//**********************************************************************
LoadDefinitions!({
  //======================================================================
  // Normally definitions disappear; the macros are expanded or have their expected effect.
  // But in a few cases (eg tabular column definitions, or LaTeX \Declarexxxx)
  // they will need declarations in the (La)TeX preamble to allow (La)TeX to process snippets
  // (eg. math) in order to create images.
  // Returning a call to this utility from Primitives will add a preamble Processing Instruction

  // TODO
  // sub AddToPreamble {
  //   my ($cs, @args) = @_;
  //   return Digest(Invocation(T_CS('\lx@add@Preamble@PI'), Invocation((ref $cs ? $cs : T_CS($cs)),
  // @args))); }

  DefConstructor!(
    "\\lx@add@Preamble@PI Undigested",
    "<?latexml preamble='#1'?>"
  );

});
