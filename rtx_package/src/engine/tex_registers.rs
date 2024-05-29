//! TeX Registers
//! 
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Registers Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  
  //======================================================================
  // Accessing Registers
  //----------------------------------------------------------------------
  // \count            iq assigns an integer to a \count register.
  // \dimen            iq assigns a <dimen> to a \dimen register.
  // \skip             iq assigns <glue> to a \skip register.
  // \muskip           iq assigns <muglue> to a \muskip register.
  // \toks             iq assigns <replacement text> to a \toks register.

  DefRegister!("\\count Number"  => Number::new(0));
  DefRegister!("\\dimen Number"  => Dimension::new(0));
  DefRegister!("\\skip Number"   => Glue::new(0));
  DefRegister!("\\muskip Number" => MuGlue::new(0));
  DefRegister!("\\toks Number"   => Tokens!());
    
  //======================================================================
  // Defining Registers, shorthands
  //----------------------------------------------------------------------
  // \countdef         c  creates a symbolic name for a \count register.
  // \dimendef         c  creates a symbolic name for a \dimen register.
  // \skipdef          c  creates a symbolic name for a \skip register.
  // \muskipdef        c  creates a symbolic name for a \muskip register.
  // \toksdef          c  creates a symbolic name for a \toks register.
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

  //======================================================================
  // Numeric Registers
  //----------------------------------------------------------------------
  // \advance          c  increases or decreases a numeric variable.
  // \multiply         c  multiplies a register by an integer.
  // \divide           c  divides a register by an integer.

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