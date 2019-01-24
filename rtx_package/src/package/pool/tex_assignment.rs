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
      getter => Some(Rc::new(move |args, state| { Some(state.lookup_number(&count).unwrap_or_default().into()) })),
      setter => Some(Rc::new(move |value, args, state| { state.assign_value(&setter_count, value, None); })));
    AfterAssignment!();
    Ok(vec![])
  });

  DefRegister!("\\catcode Number", Number::new(0.0),
    getter => Some(Rc::new(|args, state| {
      let num : f32 = args[0].to_number().value_of();
      let refchar = (num as u8) as char;
      let code : Catcode = state.lookup_catcode(refchar).unwrap_or(Catcode::OTHER);
      let code : u8 = code.into();
      Number::new(code).into()
    })),
    setter => Some(Rc::new(|value, args, state| {
      let c_char = (args[0].to_number().value_of() as u8) as char;
      let c_code = From::from(value.value_of() as u8);
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
  DefRegister!("\\hyphenchar{}", Number!((b'-')));
  DefRegister!("\\skewchar{}", Number::new(0.0)); // no idea what the default is here
});
