use package::*;

pub fn load_definitions(state: &mut State) {
  DefEnvironment!("{alltt}", "<ltx:verbatim font='#font'>#body</ltx:verbatim>",
    ConstructorOptions {
      // font => { family => 'typewriter', series => 'medium', shape => 'upright' },
      before_digest: vec![Rc::new(|stomach, state| {
        for verb_c in ['$', '&', '#', '^', '_', '%', '~'].into_iter() {
         AssignCatcode!(*verb_c, Catcode::OTHER, None, state);
        }
        AssignCatcode!(' ', Catcode::ACTIVE, None, state);
        LetI!(T_ACTIVE!(" "), T_CS!("\\space"), None, state);
        AssignValue!("PRESERVE_NEWLINES", ObjectStore::Bool(true), None, state);
        Vec::new()
      })],
      ..ConstructorOptions::default()
    },
    state);
}