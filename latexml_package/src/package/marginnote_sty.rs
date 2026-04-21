use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: marginnote.sty.ltxml
  DefConditional!("\\if@mn@verbose");

  // not documented, but in the implementation
  DeclareOption!("quiet", {
    Let!("\\if@mn@verbose", "\\iffalse");
  });
  DeclareOption!("verbose", {
    Let!("\\if@mn@verbose", "\\iftrue");
  });

  DeclareOption!("parboxrestore", {
    DefMacro!("\\mn@parboxrestore", "\\@parboxrestore");
  });
  DeclareOption!("noparboxrestore", {
    DefMacro!("\\mn@parboxrestore", None);
  });

  for option in ["fulladjust", "heightadjust", "depthadjust", "noadjust"] {
    DeclareOption!(option, None);
  }
  Digest!("\\ExecuteOptions{verbose,fulladjust,parboxrestore}")?;
  ProcessOptions!();

  DefMacro!("\\marginfont", "\\normalcolor");
  DefMacro!("\\raggedleftmarginnote", "\\raggedleft");
  DefMacro!("\\raggedrightmarginnote", "\\raggedright");

  // \marginnote: complex sub{} body — uses TokenizeInternal to build \marginpar call
  // Stub: just forward to \marginpar with the main argument
  DefMacro!("\\marginnote[]{}[]", "\\marginpar{\\mn@parboxrestore\\marginfont\\raggedrightmarginnote #2}");

  // Perl marginnote.sty.ltxml L42-46: \@mn@if@RTL dispatches at
  // expansion time — if \if@RTL is defined (LookupValue) AND currently
  // true (IfCondition), return \@firstoftwo; otherwise \@secondoftwo.
  DefMacro!("\\@mn@if@RTL", sub[_args] {
    let rtl_cs = T_CS!("\\if@RTL");
    let is_rtl = lookup_definition(&rtl_cs)?.is_some()
      && if_condition(&rtl_cs)?.unwrap_or(false);
    Ok(Tokens!(if is_rtl { T_CS!("\\@firstoftwo") } else { T_CS!("\\@secondoftwo") }))
  });

  // stubs that could do something but do not
  DefRegister!("\\marginnotevadjust" => Dimension!("0pt"));
  // Note: Perl uses LookupRegister('\textwidth') but we use 0pt as a safe default
  DefRegister!("\\marginnotetextwidth" => Dimension!("0pt"));
  Let!("\\newmarginnote", "\\newlabel");
  Let!("\\mn@lastxpos", "\\lastxpos");
  Let!("\\mn@savepos", "\\savepos");
  Let!("\\mn@pagewidth", "\\pagewidth");
  Let!("\\mn@strut", "\\strut");
  Let!("\\mn@vadjust", "\\vadjust");

  // stubs that do nothing
  DefMacro!("\\@mn@marginnote []{}",     None);
  DefMacro!("\\@mn@@marginnote []{}[]",  None);
  DefMacro!("\\@mn@@@marginnote []{}[]", None);
  DefMacro!("\\@mn@margintest",          None);
  DefMacro!("\\@mn@thispage",            None);
  DefMacro!("\\@mn@atthispage",          None);
  DefMacro!("\\@mn@currpage",            None);
  DefMacro!("\\@mn@currxpos",            None);
  DefMacro!("\\mn@vlap {}",              None);
  DefMacro!("\\mn@zbox {}",              None);

  NewCounter!("mn@abspage");
});
