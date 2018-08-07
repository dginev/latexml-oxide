use package::*;

pub fn load_definitions(core_state: &mut State) -> Result<()> {
  SetupBindingMacros!(core_state);

  DefRegister!("\\catcode Number", Number::new(0),
    getter => Some(Rc::new(|args, state| {
      let num : i32 = args[0].to_number().value_of();
      let code : Catcode = state.lookup_catcode((num as u8) as char).unwrap_or(Catcode::OTHER);
      let code : u8 = code.into();
      Number::new(code.into()).into()
    })),
    setter => Some(Rc::new(|value, args, state| {
      state.assign_catcode((args[0].to_number().value_of() as u8) as char, From::from(value.value_of() as u8), None);
    }))
  );

  Ok(())
}
