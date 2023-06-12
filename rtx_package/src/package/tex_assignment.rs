use crate::package::*;

//======================================================================
// Assignment, TeXBook Ch.24, p.275
//======================================================================
// <assignment> = <non-macro assignment> | <macro assignment>

LoadDefinitions!(outer_state, {
  //======================================================================
  // Macros
  // See Chapter 24, p.275-276
  // <macro assignment> = <definition> | <prefix><macro assignment>
  // <definition> = <def><control sequence><definition text>
  // <def> = \def | \gdef | \edef | \xdef
  // <definition text> = <register text><left brace><balanced text><right brace>
  DefPrimitive!("\\def SkipSpaces Token UntilBrace DefPlain",
    sub[stomach, (cs,params,body), state] {
      do_def(false, stomach, cs,params,body, state)?;
    },
    locked => true);
  DefPrimitive!("\\gdef SkipSpaces Token UntilBrace DefPlain",
    sub[stomach, (cs,params,body), state] {
      do_def(true, stomach, cs,params,body, state)?;
    },
    locked => true);
  DefPrimitive!("\\edef SkipSpaces Token UntilBrace DefExpanded",
    sub[stomach, (cs,params,body), state] {
      do_def(false, stomach, cs,params,body, state)?;
    },
    locked => true);
  DefPrimitive!("\\xdef SkipSpaces Token UntilBrace DefExpanded",
    sub[stomach, (cs,params,body), state] {
      do_def(true, stomach, cs,params,body, state)?;
    },
    locked => true);

  // <prefix> = \global | \long | \outer
  // See Stomach.pm & Stomach.pm
  DefPrimitive!("\\global",{ SetPrefix!("global"); }, is_prefix => true);
  DefPrimitive!("\\long",  { SetPrefix!("long");   }, is_prefix => true);
  DefPrimitive!("\\outer", { SetPrefix!("outer");  }, is_prefix => true);

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

  // Need to handle "at" too!!!
  DefPrimitive!("\\font Token SkipMatch:= SkipSpaces TeXFileName",
  sub[stomach, (cs, name_arg), state] {
    let gullet = stomach.get_gullet_mut();
    let name = name_arg.to_string();
    let props_opt = if let Some(mut props) = font::decode_fontname(&name,
      gullet.read_keyword(&["at"], state)?
        .map(|_| gullet.read_dimension(state).unwrap().pt_value(None)),
      gullet.read_keyword(&["scaled"], state)?
        .map(|_| gullet.read_number(state).unwrap().value_of() as f64 / 1000.0)) {
      props.name = Some(Cow::Owned(name));
      Some(props)
    } else { // Failed?
      let message = s!("Unrecognized font name {:?} Font switch macro {:?}
      will have no effect", name, cs.stringify());
      Info!("unexpected", name, gullet, message);
      None
    };
    gullet.skip_spaces(state)?;
    if let Some(ref props) = props_opt {
      AssignValue!(&s!("fontinfo_{}", cs.to_string()), props.clone());
    }
    DefPrimitive!(cs, None, None, font => props_opt);
  });

  // Not sure what this should be...
  DefPrimitive!("\\nullfont", None, font => {family => "nullfont"});

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

  DefPrimitive!("\\advance Variable SkipKeyword:by", sub[stomach, (var), state] {
    if let ArgWrap::RegisterDefinition(dbox) = var {
      let (defn_token, inner) = *dbox;
      let defn_token_str = defn_token.to_string();
      if !defn_token_str.is_empty() && defn_token_str != "missing" {
        let defn_opt = state.lookup_register_definition(&defn_token);
        state.local_current_token(defn_token);
        if let Some(defn) = defn_opt {
          let summand = stomach.get_gullet_mut().read_value(defn.register_type().unwrap(), state)?;
          let defn_args : Vec<ArgWrap> = inner.clone();
          let defn_value = defn.value_of(inner, state).unwrap_or_default();
          defn.set_value(defn_value.add(summand), defn_args, state);
        } else {
          let message = s!("\\advance expected a defined variable for {:?}, found no definition",
          defn_token_str);
          Error!("expected","definition", stomach, message);
        }
        state.expire_current_token();
      }
    }
  });

  DefPrimitive!("\\multiply Variable SkipKeyword:by Number", sub[stomach, (var,scale), state] {
    if let ArgWrap::RegisterDefinition(dbox) = var {
      let (varname, inner) = *dbox;
      // Upgrade: Why are the arguments used twice here? Is there a way to avoid cloning them?
      if let Some(defn) = state.lookup_register_definition(&varname) {
        let defn_args : Vec<ArgWrap> = inner.clone();
        let defn_value = defn.value_of(inner, state).unwrap_or_default();
        defn.set_value(defn_value.multiply(scale), defn_args, state);
      } else {
        let message =
          s!("\\multiply expected a defined variable for {:?}, found no definition", varname);
        Error!("expected","definition", stomach, message);
      }
    } else {
      let message = s!("\\multiply expected a Variable argument, but got nothing.");
      Error!("expected","variable", stomach, message);
    }
  });

  DefPrimitive!("\\divide Variable SkipKeyword:by Number", sub[stomach, (var,scale), state] {
    if let ArgWrap::RegisterDefinition(dbox) = var {
      let (varname, inner) = *dbox;
      // Upgrade: Why are the arguments used twice here? Is there a way to avoid cloning them?
      let defn_args : Vec<ArgWrap> = inner.clone();
      if let Some(defn) = state.lookup_register_definition(&varname) {
        let defn_value = defn.value_of(inner, state).unwrap_or_default();
        let mut denominator = scale.value_f64();
        if denominator == 0.0 {
          Error!("misdefined", scale, stomach, "Illegal \\divide by 0; assuming 1");
          denominator = 1.0;
        }
        defn.set_value(defn_value.divide(Float::new_f64(denominator)), defn_args, state);
      } else {
        let message =
          s!("\\divide expected a defined variable for {:?}, found no definition", varname);
        Error!("expected","definition", stomach, message);
      }
    } else {
      let message = s!("\\divide expected a Variable argument, but got nothing.");
      Error!("expected","variable", stomach, message);
    }
  });

  // <let assignment> = \futurelet <control sequence><token><token>
  //  | \let<control sequence><equals><one optional space><token>
  DefPrimitive!("\\let Token SkipMatch:= Skip1Space Token", sub[stomach, (token1, token2), state] {
    Let!(token1, token2);
  });

  DefPrimitive!("\\futurelet Token Token Token", sub[stomach, (cs, token1, token2), state] {
    // NOT expandable, but puts tokens back
    stomach.get_gullet_mut().unread(Tokens!(token1,token2.clone()));
    Let!(cs, token2);
  });

  // <shorthand definition> = \chardef<control sequence><equals><8bit>
  //    | \mathchardef <control sequence><equals><15bit>
  //    | <registerdef><control sequence><equals><8bit>
  // <registerdef> = \countdef | \dimendef | \skipdef | \muskipdef | toksdef

  // See below for \chardef & \mathchardef

  // DG: it's just RegisterValue actually.

  DefPrimitive!("\\countdef Token SkipMatch:=", sub[stomach, (cs), state] {
    shorthand_def(cs, "\\count", Number::new(0).into(), stomach,state)
  });

  DefPrimitive!("\\dimendef Token SkipMatch:=", sub[stomach, (cs), state] {
    shorthand_def(cs, "\\dimen", Dimension::new(0).into(), stomach,state)
  });

  DefPrimitive!("\\skipdef Token SkipMatch:=", sub[stomach, (cs), state] {
    shorthand_def(cs, "\\skip", Glue::new(0).into(), stomach,state)
  });

  DefPrimitive!("\\muskipdef Token SkipMatch:=", sub[stomach, (cs), state] {
    shorthand_def(cs, "\\muskip", MuGlue::new(0).into(), stomach,state)
  });

  DefPrimitive!("\\toksdef Token SkipMatch:=", sub[stomach, (cs), state] {
    shorthand_def(cs, "\\toks", Tokens!().into(), stomach,state)
  });

  // NOTE: Get all these handled as registers
  // <internal integer> = <integer parameter> | <special integer> | \lastpenalty
  //   | <countdef token> | \count<8bit> | <codename><8bit>
  //   | <chardef token> | <mathchardef token> | \parshape | \inputlineno
  //   | \hyphenchar<font> | \skewchar<font> | \badness

  DefRegister!("\\lastpenalty", Number::new(0), readonly => true);

  // \parshape !?!??
  DefPrimitive!("\\parshape SkipMatch:= Number", sub[stomach, (n), state] {
    let gullet = stomach.get_gullet_mut();
    for i in 0..n.value_of() {
      gullet.read_dimension(state)?;
      gullet.read_dimension(state)?;
    }
    // we _could_ conceivably store this somewhere for some attempt at stylistic purpose...
    Ok(Vec::new())
  });

  DefRegister!("\\inputlineno",Number!(0), readonly => true, getter=>sub[_args,state] {
    let stomach = state.stomach.clone();
    let stomach_mut = stomach.borrow_mut();
    let locator = stomach_mut.get_gullet().get_locator();
    Number::new(locator.map(|l| l.from_line as i64).unwrap_or(0))
  });

  DefRegister!("\\badness", Number::new(0), readonly => true);

  // <codename> = \catcode | \mathcode | \lccode | \uccode | \sfcode | \delcode
  DefRegister!("\\catcode Number", Number::new(0),
    getter => sub[args, state] {
      unpack_opt!(args => num);
      let refchar = (num.expect_number().value_of() as u8) as char;
      let code = state.lookup_catcode(refchar).unwrap_or(Catcode::OTHER);
      Number::from(code)
    },
    setter => sub[value, args, state] {
      unpack_opt!(args => num);
      let c_char = (num.expect_number().value_of() as u8) as char;
      let c_code : Catcode = From::from(value.value_of() as u8);
      state.assign_catcode(c_char, c_code, None);
    }
  );

  // Only used for active math characters, so far
  DefRegister!("\\mathcode Number", Number::new(0),
    getter => sub[args, state] {
      let ch_code   = args.remove(0).expect_number().value_of() as u8;
      let ch : char = ch_code as char;
      let code = match state.lookup_mathcode(&ch.to_string()) {
        None => ch_code,
        Some(code) => code as u8
      };
      Number!(code)
    },    // defaults to the char's code itself(?)
    setter => sub[value, args, state] {
      let ch = args.remove(0).expect_number().value_of() as u8;
      let ch : char = ch as char;
      state.assign_mathcode(ch, value.value_of() as u16, None);
    }
  );

  DefRegister!("\\sfcode Number", Number::new(0),
    getter=> sub[args, state] {
    let code = state.lookup_sfcode(args[0].value_of() as u8 as char);
      Number::new(code.unwrap_or_default() as i64)
    },
    setter => sub[value, args, state] {
      state.assign_sfcode(args[0].value_of() as u8 as char,
        value.value_of() as u16, None); });

  DefRegister!("\\lccode Number", Number::new(0),
  getter=> sub[args, state] {
    let code = state.lookup_lccode(args[0].value_of() as u8 as char);
    Number::new(code.unwrap_or_default() as i64)
  },
  setter => sub[value, args, state] {
    state.assign_lccode(args[0].value_of() as u8 as char,
      value.value_of() as u16, None);
  });

  DefRegister!("\\uccode Number", Number::new(0),
  getter=> sub[args, state] {
    let code = state.lookup_uccode(args[0].value_of() as u8 as char);
    Number::new(code.unwrap_or_default() as i64)
  },
  setter => sub[value, args, state] {
    state.assign_uccode(args[0].value_of() as u8 as char,
      value.value_of() as u16, None);
  });

  // Not used anywhere (yet)
  DefRegister!("\\delcode Number", Number::new(0),
  getter=> sub[args, state] {
    let code = state.lookup_delcode(args[0].value_of() as u8 as char);
    Number::new(code.unwrap_or_default() as i64)
  },
  setter => sub[value, args, state] {
    state.assign_delcode(args[0].value_of() as u8 as char,
      value.value_of() as u16, None);
  });

  // Remember, we're assigning a NUMBER (codepoint) to a CHARACTER!
  for letter in b'A'..=b'Z' {
    //FYI: 0x20 == 32
    outer_state.assign_lccode(letter, letter + 32, Some(Scope::Global));
    outer_state.assign_uccode(letter, letter, Some(Scope::Global));
    outer_state.assign_lccode(letter + 32, letter + 32, Some(Scope::Global));
    outer_state.assign_uccode(letter + 32, letter, Some(Scope::Global));
  }

  // Stub definitions ???
  DefRegister!("\\hyphenchar{}", Number::new(b'-' as i64));
  DefRegister!("\\skewchar{}", Number::new(0)); // no idea what the default is here

  DefMacro!("\\hyphenation GeneralText", None);
  DefMacro!("\\patterns{}", None);
});

pub fn shorthand_def(cs: Token, address_type: &str, init: RegisterValue, stomach: &mut Stomach, state: &mut State) -> Result<()> {
  state.assign_meaning(&cs, state.lookup_meaning(&TOKEN_RELAX).unwrap().into_owned(),None);
  let num = stomach.get_gullet_mut().read_number(state)?;
  let name = s!("{address_type}{}", num.value_of());
  def_register(cs, None, init,
      Some(RegisterOptions{name: Some(name), ..RegisterOptions::default()}), state);
  state.after_assignment(stomach.get_gullet_mut());
  Ok(())
}