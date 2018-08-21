use package::*;

pub fn load_definitions(core_state: &mut State) -> Result<()> {
  SetupBindingMacros!(core_state);

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

  DefRegister!("\\catcode Number", Number::new(0),
    getter => Some(Rc::new(|args, state| {
      let num : i32 = args[0].to_number().value_of();
      let code : Catcode = state.lookup_catcode((num as u8) as char).unwrap_or(Catcode::OTHER);
      let code : u8 = code.into();
      Number::new(code.into()).into()
    })),
    setter => Some(Rc::new(|value, args, state| {
      let c_char = (args[0].to_number().value_of() as u8) as char;
      let c_code = From::from(value.value_of() as u8);
      state.assign_catcode(c_char, c_code, None);
    }))
  );

  Ok(())
}
