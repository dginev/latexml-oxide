use crate::prelude::*;

//======================================================================
// Assignment, TeXBook Ch.24, p.275
//======================================================================
// <assignment> = <non-macro assignment> | <macro assignment>

LoadDefinitions!({
  //======================================================================
  // Non-Macro assignments; TeXBook Ch.24, pp 276--277
  // <non-macro assignment> = <simple assignment> | \global <non-macro assignment>

  // <filler> = <optional spaces> | <filler>\relax<optional spaces>
  // <general text> = <filler>{<balanced text><right brace>

  // <simple assignment> = <variable assignment> | <arithmetic>
  //    | <code assignment> | <let assignment> | <shorthand definition>
  //    | <fontdef token> | <family assignment> | <shape assignment>
  //    | \read <number> to <optional spaces><control sequence>
  //    | \setbox<8bit><equals><filler><box>
  //    | \font <control sequence><equals><file name><at clause>
  //    | <global assignment>
  // <variable assignment> = <integer variable><equals><number>
  //    | <dimen variable><equals><dimen>
  //    | <glue variable><equals><dimen>
  //    | <muglue variable><equals><muglue>
  //    | <token variable><equals><general text>
  //    | <token variable><equals><token variable>
  // <at clause> = at <dimen> | scaled <number> | <optional spaces>
  // <code assignment> = <codename><8bit><equals><number>

  DefRegister!("\\count Number"  => Number::new(0));
  DefRegister!("\\dimen Number"  => Dimension::new(0));
  DefRegister!("\\skip Number"   => Glue::new(0));
  DefRegister!("\\muskip Number" => MuGlue::new(0));
  DefRegister!("\\toks Number"   => Tokens!());

  // <integer variable> = <integer parameter> | <countdef token> | \count<8bit>
  // <dimen var> = <dimen parameter> | <dimendef token> | \dimen<8bit>
  // <glue variable> = <glue parameter> | <skipdef token> | \skip<8bit>
  // <muglue variable> = <muglue parameter> | <muskipdef token> | \muskip<8bit>

  // <arithmetic> = \advance <integer variable><optional by><number>
  //    | \advance <dimen variable><optional by><dimen>
  //    | \advance <glue variable><optional by><glue>
  //    | \advance <muglue variable><optional by><muglue>
  //    | \multiply <numeric variable><optional by><number>
  //    | \divide <numeric variable><optional by><number>

  DefPrimitive!("\\advance Variable SkipKeyword:by", sub[(var)] {
    if let ArgWrap::RegisterDefinition(dbox) = var {
      let (defn_token, inner) = *dbox;
      let defn_token_str = defn_token.to_string();
      if !defn_token_str.is_empty() && defn_token_str != "missing" {
        let defn_opt = state::lookup_register_definition(&defn_token);
        local_current_token(defn_token);
        if let Some(defn) = defn_opt {
          let summand = gullet::read_value(defn.register_type().unwrap())?;
          let defn_args : Vec<ArgWrap> = inner.clone();
          let defn_value = defn.value_of(inner).unwrap_or_default();
          defn.set_value(defn_value.add(summand), None, defn_args);
        } else {
          let message = s!("\\advance expected a defined variable for {:?}, found no definition",
          defn_token_str);
          Error!("expected","definition", message);
        }
        expire_current_token();
      }
    }
  });

  DefPrimitive!("\\multiply Variable SkipKeyword:by Number", sub[(var,scale)] {
    if let ArgWrap::RegisterDefinition(dbox) = var {
      let (varname, inner) = *dbox;
      // Upgrade: Why are the arguments used twice here? Is there a way to avoid cloning them?
      if let Some(defn) = state::lookup_register_definition(&varname) {
        let defn_args : Vec<ArgWrap> = inner.clone();
        let defn_value = defn.value_of(inner).unwrap_or_default();
        defn.set_value(defn_value.multiply(scale), None, defn_args);
      } else {
        let message =
          s!("\\multiply expected a defined variable for {:?}, found no definition", varname);
        Error!("expected","definition", message);
      }
    } else {
      let message = s!("\\multiply expected a Variable argument, but got nothing.");
      Error!("expected","variable", message);
    }
  });

  DefPrimitive!("\\divide Variable SkipKeyword:by Number", sub[(var,scale)] {
    if let ArgWrap::RegisterDefinition(dbox) = var {
      let (varname, inner) = *dbox;
      // Upgrade: Why are the arguments used twice here? Is there a way to avoid cloning them?
      let defn_args : Vec<ArgWrap> = inner.clone();
      if let Some(defn) = state::lookup_register_definition(&varname) {
        let defn_value = defn.value_of(inner).unwrap_or_default();
        let mut denominator = scale.value_f64();
        if denominator == 0.0 {
          Error!("misdefined", scale, "Illegal \\divide by 0; assuming 1");
          denominator = 1.0;
        }
        defn.set_value(defn_value.divide(Float::new_f64(denominator)), None, defn_args);
      } else {
        let message =
          s!("\\divide expected a defined variable for {:?}, found no definition", varname);
        Error!("expected","definition", message);
      }
    } else {
      let message = s!("\\divide expected a Variable argument, but got nothing.");
      Error!("expected","variable", message);
    }
  });

  // <shorthand definition> = \chardef<control sequence><equals><8bit>
  //    | \mathchardef <control sequence><equals><15bit>
  //    | <registerdef><control sequence><equals><8bit>
  // <registerdef> = \countdef | \dimendef | \skipdef | \muskipdef | toksdef

  // See below for \chardef & \mathchardef

  // DG: it's just RegisterValue actually.

  DefPrimitive!("\\countdef Token SkipMatch:=", sub[(cs)] {
    shorthand_def(cs, "\\count", Number::new(0).into())
  });

  DefPrimitive!("\\dimendef Token SkipMatch:=", sub[(cs)] {
    shorthand_def(cs, "\\dimen", Dimension::new(0).into())
  });

  DefPrimitive!("\\skipdef Token SkipMatch:=", sub[(cs)] {
    shorthand_def(cs, "\\skip", Glue::new(0).into())
  });

  DefPrimitive!("\\muskipdef Token SkipMatch:=", sub[(cs)] {
    shorthand_def(cs, "\\muskip", MuGlue::new(0).into())
  });

  DefPrimitive!("\\toksdef Token SkipMatch:=", sub[(cs)] {
    shorthand_def(cs, "\\toks", Tokens!().into())
  });

  // NOTE: Get all these handled as registers
  // <internal integer> = <integer parameter> | <special integer> | \lastpenalty
  //   | <countdef token> | \count<8bit> | <codename><8bit>
  //   | <chardef token> | <mathchardef token> | \parshape | \inputlineno
  //   | \hyphenchar<font> | \skewchar<font> | \badness

  DefRegister!("\\lastpenalty", Number::new(0), readonly => true);

  // \parshape !?!??
  DefPrimitive!("\\parshape SkipMatch:= Number", sub[(n)] {
    for _ in 0..n.value_of() {
      gullet::read_dimension()?;
      gullet::read_dimension()?;
    }
    // we _could_ conceivably store this somewhere for some attempt at stylistic purpose...
    Ok(Vec::new())
  });

  DefRegister!("\\badness", Number::new(0), readonly => true);
});

/// Note that these define a "shorthand" for eg. \count123, but are NOT macros!
pub fn shorthand_def(cs: Token, address_type: &str, init: RegisterValue) -> Result<()> {
  // Let w/o AfterAssign
  let relax_meaning = lookup_meaning(&TOKEN_RELAX).unwrap();
  assign_meaning(&cs, relax_meaning,None);
  // define
  let num = gullet::read_number()?;
  let address = s!("{address_type}{}", num.value_of());
  let options = Some(RegisterOptions{
    address: Some(address),
    ..RegisterOptions::default()});
  def_register(cs, None, init, options)?;
  after_assignment();
  Ok(())
}