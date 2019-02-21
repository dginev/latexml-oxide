use crate::package::*;

//======================================================================
// Assignment, TeXBook Ch.24, p.275
//======================================================================
// <assignment> = <non-macro assignment> | <macro assignment>
LoadDefinitions!(state, {
  //======================================================================
  // Macros
  // See Chapter 24, p.275-276
  // <macro assignment> = <definition> | <prefix><macro assignment>
  // <definition> = <def><control sequence><definition text>
  // <def> = \def | \gdef | \edef | \xdef
  // <definition text> = <register text><left brace><balanced text><right brace>
  DefPrimitiveI!("\\def SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(false, false, stomach, args, state)
    },
    locked => true
  );
  DefPrimitiveI!("\\gdef SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(true, false, stomach, args, state)
    },
    locked => true
  );
  DefPrimitiveI!("\\edef SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(false, true, stomach, args, state)
    },
    locked => true
  );
  DefPrimitiveI!("\\xdef SkipSpaces Token UntilBrace {}", |stomach, args, state| {
      do_def(true, true, stomach, args, state)
    },
    locked => true
  );

  // <prefix> = \global | \long | \outer
  // See Stomach.pm & Stomach.pm
  DefPrimitive!("\\global",sub { SetPrefix!("global"); }, is_prefix => true);
  DefPrimitive!("\\long",  sub { SetPrefix!("long");   }, is_prefix => true);
  DefPrimitive!("\\outer", sub { SetPrefix!("outer");  }, is_prefix => true);

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
  DefPrimitive!("\\font Token SkipMatch:= SkipSpaces TeXFileName", sub[stomach, args, state] {
    unpack_to_token!(args => cs, name);
    let gullet = stomach.get_gullet_mut();
    let name = name.to_string();
    if let Some(props) = font::decode_fontname(&name,
         gullet.read_keyword(&["at"], state)?.map(|_| gullet.read_dimension(state).unwrap().pt_value(None)),
         gullet.read_keyword(&["scaled"], state)?.map(|_| gullet.read_number(state).unwrap().value_of() / 1000.0)) {

      gullet.skip_spaces(state);
      AssignValue!(&s!("fontinfo_{}", cs.to_string()), props.clone());
      DefPrimitiveII!(cs, None, None, font => Some(props));
    } else {    // Failed?
      info!(target: &s!("unexpected:{}", name),
        "Unrecognized font name {:?} Font switch macro {:?} will have no effect", name, cs.to_string());
    }
  });

  // Not sure what this should be...
  DefPrimitive!("\\nullfont", None, font => Font!(family => "nullfont"));

  DefRegister!("\\count Number"  => Number::new(0.0));
  DefRegister!("\\dimen Number"  => Dimension::new(0.0));
  DefRegister!("\\skip Number"   => Glue::new(0.0));
  DefRegister!("\\muskip Number" => MuGlue::new(0.0));
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

  DefPrimitive!("\\advance Variable SkipKeyword:by", sub[stomach, args, state] {
    unpack!(args => var);
    // TODO: Variable type unpacking seems to require special INFRA again...
    let mut var_tokens = var.unlist();
    if !var_tokens.is_empty() {
      let defn_token = var_tokens.remove(0);
      if defn_token.to_string() != "missing" {
        let defn_opt = state.lookup_register_definition(&defn_token);
        let defn_token_rc = Rc::new(defn_token);
        state.current_token = Some(Rc::clone(&defn_token_rc));
        if let Some(defn) = defn_opt {
          let summand = stomach.get_gullet_mut().read_value(defn.register_type().unwrap(), state)?;
          let defn_args : Vec<Tokens> = var_tokens.iter().map(|a| Tokens!(a.clone())).collect();
          let defn_value = defn.value_of(var_tokens, state).unwrap_or_default();
          defn.borrow_mut().set_value(defn_value.add(summand), defn_args, state);
        } else {
          error!(target: "expected:definition", "\\advance expected a defined variable for {:?}, found no definition", defn_token_rc);
        }
      }
    }
  });

  DefPrimitive!("\\multiply Variable SkipKeyword:by Number", sub[stomach, args, state] {
    unpack!(args => var, scale);
    if !var.is_empty() {
      let mut args = var.unlist();
      let varname = args.remove(0);
      // TODO: Why are the arguments used twice here? Is there a way to avoid cloning them?
      let defn_args : Vec<Tokens> = args.iter().map(|a| Tokens!(a.clone())).collect();
      if let Some(defn) = state.lookup_register_definition(&varname) {
        // TODO: We need a strategy for obtaining the Variable here, to be able to perform the primitive operations
        let defn_value = defn.value_of(args, state).unwrap_or_default();
        defn.borrow_mut().set_value(defn_value.multiply(scale.value_of(Vec::new(), state).unwrap_or_default()), defn_args, state);
      } else {
        error!(target: "expected:definition", "\\multiply expected a defined variable for {:?}, found no definition", varname);
      }
    } else {
      error!(target: "expected:variable", "\\multiply expected a Variable argument, but got nothing.");
    }
  });

  DefPrimitive!("\\divide Variable SkipKeyword:by Number", sub[stomach, args, state] {
    unpack!(args => var, scale);
    // return () unless $var;
    // let (defn, @args) = @$var;
    // my $denom = $scale->valueOf;
    // if ($denom == 0) {
    //   Error('misdefined', $scale, $stomach, "Illegal \\divide by 0; assuming 1");
    //   $denom = 1; }
    // $defn->setValue($defn->valueOf(@args)->divide($denom), @args); });
    unimplemented!();
    ()
  });

  // <let assignment> = \futurelet <control sequence><token><token>
  //  | \let<control sequence><equals><one optional space><token>
  DefPrimitive!("\\let Token SkipMatch:= Skip1Space Token", sub[stomach, args, state] {
   unpack_to_token!(args => token1, token2);
   LetI!(&token1, token2);
   Ok(Vec::new())
  });

  DefMacro!("\\futurelet Token Token Token", sub[gullet, args, state] {
      unpack_to_token!(args => cs, token1, token2);
      LetI!(&cs, token2.clone());
      Ok(Tokens!(token1, token2))
  });

  // <shorthand definition> = \chardef<control sequence><equals><8bit>
  //    | \mathchardef <control sequence><equals><15bit>
  //    | <registerdef><control sequence><equals><8bit>
  // <registerdef> = \countdef | \dimendef | \skipdef | \muskipdef | toksdef

  // See below for \chardef & \mathchardef

  DefPrimitive!("\\countdef Token SkipMatch:= Number", sub[stomach, args, inner_state] {
    unpack_to_token!(args => cs, num);
    let count = s!("\\count{}", num.to_number().value_of());
    let setter_count = count.clone();
    DefRegisterI!(cs, None, Number::new(0.0),
      getter => getter!(args, state, { state.lookup_number(&count).unwrap_or_default() }),
      setter => setter!(value, args, state, { state.assign_value(&setter_count, value, None); }));
    AfterAssignment!();
    Ok(vec![])
  });

  DefPrimitive!("\\dimendef Token SkipMatch:= Number", sub[stomach,args,state] {
    unpack_to_token!(args=> cs, num);
    let dimen = s!("\\dimen{}", num.to_number().value_of());
    let dimen2 = dimen.clone();
    DefRegisterI!(cs, None, Dimension::new(0.0),
      getter => getter!(args, state, { state.lookup_dimension(&dimen).unwrap_or_default() }),
      setter => setter!(value, args, state, { state.assign_value(&dimen2, value, None); })
    );
    AfterAssignment!();
  });

  DefPrimitive!("\\skipdef Token SkipMatch:= Number", sub[stomach,args,state] {
    unpack_to_token!(args=> cs, num);
    let skip = s!("\\skip{}", num.to_number().value_of());
    let skip2 = skip.clone();
    DefRegisterI!(cs, None, Glue::new(0.0),
      getter => getter!(args, state, { state.lookup_glue(&skip).unwrap_or_default() }),
      setter => setter!(value, args, state, { state.assign_value(&skip2, value, None); })
    );
    AfterAssignment!();
  });

  DefPrimitive!("\\muskipdef Token SkipMatch:= Number", sub[stomach,args,state] {
    unpack_to_token!(args=> cs, num);
    // my $muglue = '\muskip' . $num->valueOf;
    // DefRegisterI($cs, undef, MuGlue(0),
    //   getter => sub { LookupValue($muglue) || MuGlue(0); },
    //   setter => sub { AssignValue($muglue => $_[0]); });
    AfterAssignment!();
    unimplemented!();
    ()
  });

  DefPrimitive!("\\toksdef Token SkipMatch:= Number", sub[stomach,args,state] {
    unpack_to_token!(args=> cs, num);
    // my $toks = '\toks' . $num->valueOf;
    // DefRegisterI($cs, undef, Tokens(),
    //   getter => sub { LookupValue($toks) || Tokens(); },
    //   setter => sub { AssignValue($toks => $_[0]); });
    AfterAssignment!();
    unimplemented!();
    ()
  });

  // NOTE: Get all these handled as registers
  // <internal integer> = <integer parameter> | <special integer> | \lastpenalty
  //   | <countdef token> | \count<8bit> | <codename><8bit>
  //   | <chardef token> | <mathchardef token> | \parshape | \inputlineno
  //   | \hyphenchar<font> | \skewchar<font> | \badness

  DefRegister!("\\lastpenalty", Number::new(0.0), readonly => true);

  // \parshape !?!??
  DefPrimitive!("\\parshape SkipMatch:= Number", sub[stomach, args, state] {
    unpack_to_token!(args => n);
    // $n = $n->valueOf;
    // my $gullet = $stomach->getGullet;
    // for (my $i = 0 ; $i < $n ; $i++) {
    //   $gullet->readDimension; $gullet->readDimension; }
    // // we _could_ conceivably store this somewhere for some attempt at stylistic purpose...
    unimplemented!();
    ()
  });

  //DefRegister('\inputlineno',Number(0),
  //            readonly=>1,
  //            getter=>{ Number($stomach->getGullet->getMouth????? ->lineno); });

  DefRegister!("\\badness", Number::new(0.0), readonly => true);

  // <codename> = \catcode | \mathcode | \lccode | \uccode | \sfcode | \delcode
  DefRegister!("\\catcode Number", Number::new(0.0),
    getter => getter!(args, state, {
      let num : f32 = args[0].to_number().value_of();
      let refchar = (num as u8) as char;
      let code : Catcode = state.lookup_catcode(refchar).unwrap_or(Catcode::OTHER);
      let code : u8 = code.into();
      Number::new(code)
    }),
    setter => setter!(value, args, state, {
      let c_char = (args[0].to_number().value_of() as u8) as char;
      let c_code = From::from(value.value_of() as u8);
      state.assign_catcode(c_char, c_code, None);
    })
  );

  // Only used for active math characters, so far
  DefRegister!("\\mathcode Number", Number::new(0.0),
    getter => getter!(args, state, {
      let ch_code   = args[0].to_number().value_of() as u8;
      let ch : char = ch_code as char;
      let code = match state.lookup_mathcode(&ch.to_string()) {
        None => ch_code,
        Some(code) => code as u8
      };
      Number::new(f32::from(code))
    }),    // defaults to the char's code itself(?)
    setter => setter!(value, args, state, {
      let ch = args[0].to_number().value_of() as u8;
      let ch : char = ch as char;
      state.assign_mathcode(ch, value.value_of() as u16, None);
    })
  );

  DefRegister!("\\sfcode Number", Number::new(0.0),
    getter=>getter!(args, state, { unimplemented!(); () }),
    // my $code = $STATE->lookupSFcode(chr($_[0]->valueOf));
    //  Number(defined $code ? $code : 0); },
    setter => setter!(value, args, state, { unimplemented!(); ()
      //$STATE->assignSFcode(chr($_[1]->valueOf) => $_[0]->valueOf);
      })
  );
  DefRegister!("\\lccode Number", Number::new(0.0),
    getter=>getter!(args, state, { unimplemented!(); () }),
      // my $code = $STATE->lookupLCcode(chr($_[0]->valueOf));
      // Number(defined $code ? $code : 0); },
    setter => setter!(value, args, state, { unimplemented!(); ()
      //$STATE->assignLCcode(chr($_[1]->valueOf) => $_[0]->valueOf);
      })
  );
  DefRegister!("\\uccode Number", Number::new(0.0),
    getter=>getter!(args, state, { unimplemented!(); () }),
      // my $code = $STATE->lookupUCcode(chr($_[0]->valueOf));
      // Number(defined $code ? $code : 0); },
    setter => setter!(value, args, state, { unimplemented!(); ()
      //$STATE->assignUCcode(chr($_[1]->valueOf) => $_[0]->valueOf);
    })
  );
  // Not used anywhere (yet)
  DefRegister!("\\delcode Number", Number::new(0.0),
    getter=>getter!(args, state, { unimplemented!(); () }),
      // my $code = $STATE->lookupDelcode(chr($_[0]->valueOf));
      // Number(defined $code ? $code : 0); },
    setter => setter!(value, args, state, { unimplemented!(); ()
      //$STATE->assignDelcode(chr($_[1]->valueOf) => $_[0]->valueOf);
    })
  );

  // Remember, we're assigning a NUMBER (codepoint) to a CHARACTER!
  for letter in b'A'..=b'Z' {
    state.assign_lccode(letter, letter + 20, Scope::Global);
    state.assign_uccode(letter, letter, Scope::Global);
    state.assign_lccode(letter + 20, letter + 20, Scope::Global);
    state.assign_uccode(letter + 20, letter, Scope::Global);
  }

  // Stub definitions ???
  DefRegister!("\\hyphenchar{}", Number!((b'-')));
  DefRegister!("\\skewchar{}", Number::new(0.0)); // no idea what the default is here

  DefMacro!("\\hyphenation GeneralText", "");
});
