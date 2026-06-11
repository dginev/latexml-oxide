use latexml_core::binding::content::convert_twoopt_args;

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl twoopt.sty.ltxml L20-52. Builds Parameters with two optional args
  // + plain args, then installs the CS via DefMacroI. Matches Perl's
  // `convert2optArgs` helper + DefPrimitive bodies exactly.

  // \newcommandtwoopt[*]\cs[nargs][opt1][opt2]{body}
  DefPrimitive!("\\newcommandtwoopt OptionalMatch:* DefToken [Number][][]{}",
    sub[(_star, cs, nargs, opt1, opt2, body)] {
      if !IsDefinable!(&cs) {
        if !has_value(&s!("{}:locked", cs.to_string())) {
          let msg = s!("Ignoring redefinition (\\newcommandtwoopt) of {}", cs.stringify());
          Info!("ignore", cs, msg);
        }
        return Ok(vec![]);
      }
      let n = nargs.value_of() as usize;
      let params = convert_twoopt_args(n, opt1, opt2)?;
      DefMacro!(cs, params, body);
    });

  DefPrimitive!("\\renewcommandtwoopt OptionalMatch:* DefToken [Number][][]{}",
    sub[(_star, cs, nargs, opt1, opt2, body)] {
      let n = nargs.value_of() as usize;
      let params = convert_twoopt_args(n, opt1, opt2)?;
      DefMacro!(cs, params, body);
    });

  DefPrimitive!("\\providecommandtwoopt OptionalMatch:* DefToken [Number][][]{}",
    sub[(_star, cs, nargs, opt1, opt2, body)] {
      if !IsDefinable!(&cs) { return Ok(vec![]); }
      let n = nargs.value_of() as usize;
      let params = convert_twoopt_args(n, opt1, opt2)?;
      DefMacro!(cs, params, body);
    });
});
