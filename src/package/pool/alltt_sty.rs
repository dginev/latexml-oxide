use package::*;

 pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  DefEnvironment!("{alltt}", "<ltx:verbatim font='#font'>#body</ltx:verbatim>",
    font => Font!(family => "typewriter", series => "medium", shape => "upright"),
    before_digest => sub!(|stomach, state| {
      for c in &['$', '&', '#', '^', '_', '%', '~'] {
       AssignCatcode_F!(*c, Catcode::OTHER, None, state);
      }
      AssignCatcode_F!(' ', Catcode::ACTIVE, None, state);
      LetI_F!(&T_ACTIVE!(" "), T_CS!("\\space"), None, state);
      AssignValue_F!("PRESERVE_NEWLINES", ObjectStore::Bool(true), None, state);
      Ok(Vec::new())
    }));

  Ok(())
}
