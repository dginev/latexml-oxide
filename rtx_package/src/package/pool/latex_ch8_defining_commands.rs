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
      unpack!(args => star, cs, nargs, opt, body);
      let cs_token: Token = cs.into();
      let nargs_token: Token = nargs.into();
      let nargs = nargs_token.to_number().value_of() as usize;
      // if (!isDefinable(cs)) {
      //   Info('ignore', $cs, $stomach,
      //     "Ignoring redefinition (\\newcommand) of '" . Stringify($cs) . "'")
      //     unless LookupValue(ToString($cs) . ':locked');
      //   return; }
      let opt = if opt.is_empty() { None } else { Some(opt) };
      let macro_args = convert_latex_args(nargs, opt, state)?;
      DefMacroI!(cs_token, macro_args, body);
    })
  );
});
