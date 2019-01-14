use crate::package::*;

LoadDefinitions!(state, {
  //======================================================================
  // Assignment, TeXBook Ch.24, p.275
  //======================================================================
  // <assignment> = <non-macro assignment> | <macro assignment>

  //======================================================================
  // Macros
  // See Chapter 24, p.275-276
  // <macro assignment> = <definition> | <prefix><macro assignment>
  // <definition> = <def><control sequence><definition text>
  // <def> = \def | \gdef | \edef | \xdef
  // <definition text> = <register text><left brace><balanced text><right brace>

  fn parse_def_parameters(cs: &Token, params_in: Tokens, state: &mut State) -> Result<Option<Parameters>> {
    let mut tokens: VecDeque<Token> = VecDeque::from(params_in.unlist());
    // Now, recognize parameters and delimiters.
    let mut params = Vec::new();
    let mut n = 0;
    while let Some(mut t) = tokens.pop_front() {
      if t.get_catcode() == Catcode::PARAM {
        if tokens.is_empty() {
          // Special case: lone # NOT following a numbered parameter
          // Note that we require a { to appear next, but do NOT read it!
          params.push(Parameter::new("RequireBrace", "RequireBrace", state)?);
        } else {
          n += 1;
          t = tokens.pop_front().unwrap();
          // TODO: Double-check we're not missing cases from the original:
          //       ($n == (ord($t->getString) - ord('0'))
          let t_num = t.get_string().parse::<i32>().unwrap_or(-1);
          if t_num != n {
            fatal!(ParamSpec, Expected, s!("Parameters for {:?} not in order in {:?}", cs, params));
          }
          // Check for delimiting text following the parameter #n
          let mut delim = Vec::new();
          let mut pc = Catcode::MARKER; // throwaway initial val
          let mut cc;
          while !tokens.is_empty() && (tokens.front().unwrap().get_catcode() != Catcode::PARAM) {
            let d = tokens.pop_front().unwrap();
            cc = d.get_catcode();
            if !(cc == pc && cc == Catcode::SPACE) {
              // BUT collapse whitespace!
              delim.push(d);
            }
            pc = cc;
          }
          // Found text that marks the end of the parameter
          if !delim.is_empty() {
            let expected = Tokens::new(delim);
            params.push(
              Parameter {
                name: s!("Until"),
                spec: s!("Until:{}", expected),
                extra: expected.into(),
                ..Parameter::default()
              }
              .init(state)?,
            );
          } else if tokens.len() == 1 && tokens.front().unwrap().get_catcode() == Catcode::PARAM {
            // Special case: trailing sole # => delimited by next opening brace.
            tokens.pop_front();
            params.push(Parameter::new("UntilBrace", "UntilBrace", state)?);
          } else {
            // Nothing? Just a plain parameter.
            params.push(Parameter::new("Plain", "{}", state)?);
          }
        }
      } else {
        // Initial delimiting text is required.
        let mut lit: Vec<Token> = vec![t];
        while !tokens.is_empty() && (tokens.front().unwrap().get_catcode() != Catcode::PARAM) {
          lit.push(tokens.pop_front().unwrap());
        }
        let expected = Tokens::new(lit);
        params.push(
          Parameter {
            name: s!("Match"),
            spec: s!("Match:{}", expected),
            extra: expected.into(),
            novalue: true,
            ..Parameter::default()
          }
          .init(state)?,
        );
      }
    }
    // return (@params ? LaTeXML::Core::Parameters->new(@params) : undef);
    if params.is_empty() {
      Ok(None)
    } else {
      Ok(Some(Parameters { params }))
    }
  }

  fn do_def(globally: bool, expanded: bool, stomach: &mut Stomach, args: Vec<Tokens>, state: &mut State) -> Result<Vec<Digested>> {
    unpack!(args => cs, params, body);
    // ensure params is empty if it contains only the default token
    // TODO: is this a flaw of parameter parsing?
    let params = if params.tokens == MOCK_TOKENS.tokens { Tokens!() } else { params };
    let cs: Token = cs.into();
    let paramlist = parse_def_parameters(&cs, params, state)?;
    if expanded {
      state.noexpand_the = true;
      let gullet = stomach.get_gullet_mut();
      body = Expand!(body, gullet, state);
    }
    let scope = if globally { Some(Scope::Global) } else { None };
    state.install_definition(
      Expandable {
        cs,
        paramlist,
        expansion: body.into(),
        ..Expandable::default()
      },
      scope,
    );
    AfterAssignment!(state);
    Ok(Vec::new())
  }

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
  DefPrimitive!("\\global",sub[stomach, args, state] { state.set_prefix("global");  Ok(vec![])}, is_prefix => true);
  DefPrimitive!("\\long",  sub[stomach, args, state] { state.set_prefix("long");    Ok(vec![])}, is_prefix => true);
  DefPrimitive!("\\outer", sub[stomach, args, state] { state.set_prefix("outer");   Ok(vec![])}, is_prefix => true);

  // <let assignment> = \futurelet <control sequence><token><token>
  //  | \let<control sequence><equals><one optional space><token>
  DefPrimitive!("\\let Token SkipMatch:= Skip1Space Token", sub[stomach, args, state] {
   unpack_to_token!(args => token1, token2);
   state.let_i(&token1, token2, None);
   Ok(Vec::new())
  });

  DefMacro!("\\futurelet Token Token Token", sub[gullet, args, state] {
      unpack_to_token!(args => cs, token1, token2);
      state.let_i(&cs, token2.clone(), None);
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
    DefRegister!(&cs.get_cs_name(), Number::new(0.0), inner_state,
      getter => Some(Rc::new(move |args, state| { Some(state.lookup_number(&count).unwrap_or_default().into()) })),
      setter => Some(Rc::new(move |value, args, state| { state.assign_value(&setter_count, value, None); })));
    AfterAssignment!(inner_state);
    Ok(vec![])
  });

  DefRegister!("\\catcode Number", Number::new(0.0),
    getter => Some(Rc::new(|args, state| {
      let num : f32 = args[0].to_number().value_of();
      let refchar = (num as u8) as char;
      info!("-- looking up {:?}", refchar);
      let code : Catcode = state.lookup_catcode(refchar).unwrap_or(Catcode::OTHER);
      let code : u8 = code.into();
      info!("-- code is: {:?}", code);
      Number::new(code).into()
    })),
    setter => Some(Rc::new(|value, args, state| {
      let c_char = (args[0].to_number().value_of() as u8) as char;
      info!("-- c_char: {:?}", c_char);
      let c_code = From::from(value.value_of() as u8);
      info!("-- c_code: {:?}", c_code);
      state.assign_catcode(c_char, c_code, None);
    }))
  );

  // Only used for active math characters, so far
  DefRegister!("\\mathcode Number", Number::new(0.0),
    getter => Some(Rc::new(|args, state| {
      let ch_code   = args[0].to_number().value_of() as u8;
      let ch : char = ch_code as char;
      let code = match state.lookup_mathcode(&ch.to_string()) {
        None => ch_code,
        Some(code) => code as u8
      };
      Some(Number::new(f32::from(code)).into())
    })),    // defaults to the char's code itself(?)
    setter => Some(Rc::new(|value, args, state| {
      let ch = args[0].to_number().value_of() as u8;
      let ch : char = ch as char;
      state.assign_mathcode(ch, value.value_of() as usize, None);
    }))
  );

  // Stub definitions ???
  DefRegister!("\\hyphenchar{}", Number!(('-' as u8)));
  DefRegister!("\\skewchar{}", Number::new(0.0)); // no idea what the default is here
});
