use package::*;

pub fn load_definitions(state: &mut State) {
  SetupBindingMacros!(state);

  DefEnvironment!("{alltt}", "<ltx:verbatim font='#font'>#body</ltx:verbatim>",
    before_digest => vec![Rc::new(|stomach, state| {
      for verb_c in &['$', '&', '#', '^', '_', '%', '~'] {
       AssignCatcode!(*verb_c, Catcode::OTHER, None, state);
      }
      AssignCatcode!(' ', Catcode::ACTIVE, None, state);
      LetI!(T_ACTIVE!(" "), T_CS!("\\space"), None, state);
      AssignValue!("PRESERVE_NEWLINES", ObjectStore::Bool(true), None, state);
      Vec::new()
    })]
  );
}