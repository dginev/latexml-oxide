use rtx_package::package::*;

pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  DefMacro!("\\hw", "hello world!");

  Ok(())
}
