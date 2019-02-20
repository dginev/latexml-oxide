use crate::package::*;
LoadDefinitions!(state, {
  //**********************************************************************
  // C.8 Definitions, Numbering and Programming
  //**********************************************************************

  //======================================================================
  // C.8.1 Defining Commands
  //======================================================================

  // DefMacro('\@tabacckludge {}', '\csname\string#1\endcsname');

  DefPrimitiveI!(
    "\\newcommand OptionalMatch:* DefToken [Number][]{}",
    primitiveproc!(stomach, args, state, {
      // my ($stomach, $star, $cs, $nargs, $opt, $body) = @_;
      let star = &args[0];
      let cs: Token = (&args[1]).into();
      let nargs = &args[2];
      let opt = &args[3];
      let body = args[4].clone();

      // if (!isDefinable(cs)) {
      //   Info('ignore', $cs, $stomach,
      //     "Ignoring redefinition (\\newcommand) of '" . Stringify($cs) . "'")
      //     unless LookupValue(ToString($cs) . ':locked');
      //   return; }

      // TODO: convertLaTeXArgs($nargs, $opt)
      DefMacroI!(cs.clone(), None, body, state);
    })
  );
});
