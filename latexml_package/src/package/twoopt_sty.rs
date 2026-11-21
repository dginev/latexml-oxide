use latexml_core::binding::content::convert_twoopt_args;
use latexml_engine::latex_constructs::define_optional_internal;

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
      define_twoopt(&cs, nargs.value_of() as usize, opt1, opt2, body)?;
    });

  DefPrimitive!("\\renewcommandtwoopt OptionalMatch:* DefToken [Number][][]{}",
    sub[(_star, cs, nargs, opt1, opt2, body)] {
      define_twoopt(&cs, nargs.value_of() as usize, opt1, opt2, body)?;
    });

  DefPrimitive!("\\providecommandtwoopt OptionalMatch:* DefToken [Number][][]{}",
    sub[(_star, cs, nargs, opt1, opt2, body)] {
      if !IsDefinable!(&cs) { return Ok(vec![]); }
      define_twoopt(&cs, nargs.value_of() as usize, opt1, opt2, body)?;
    });
});

/// twoopt.sty:51-55: `\cs` is `\newcommand\cs[1][{d1}]{…}`, the call `\@protected@testopt\cs\\cs{d1}`, whose
/// internal `\\cs` here reads both optional arguments and the rest itself (sect08.rs `define_optional_internal`).
fn define_twoopt(
  cs: &Token,
  nargs: usize,
  opt1: Option<Tokens>,
  opt2: Option<Tokens>,
  body: Tokens,
) -> Result<()> {
  let first = opt1.is_some();
  let params = convert_twoopt_args(nargs, opt1, opt2)?;
  let internal = if first {
    define_optional_internal(cs, params.clone(), &body, false, None)?
  } else {
    None
  };
  let params = match internal {
    Some(internal) => params.map(|params| params.with_testopt_call(TestoptCall::kernel(internal))),
    None => params,
  };
  DefMacro!(*cs, params, body);
  Ok(())
}
