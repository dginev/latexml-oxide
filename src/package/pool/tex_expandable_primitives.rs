use package::*;

pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  DefConditional!("\\ifx Token Token", gullet, args, inner_state, {
    if let Some(token1) = args[0].tokens.first() {
      if let Some(token2) = args[1].tokens.first() {
        let xequals = XEquals!(token1, token2, inner_state);
        Ok(xequals)
      } else {
        Ok(false)
      }
    } else {
      Ok(false)
    }
  });

  Ok(())
}
