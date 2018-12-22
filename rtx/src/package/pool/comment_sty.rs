use crate::package::*;

pub fn load_definitions(outer_state: &mut State) -> Result<()> {
  SetupBindingMacros!(outer_state);

  //**********************************************************************
  // Define \name and \begin{name} to start an ignored section
  // until \endname or \end{name}, respectively
  let define_excluded = primitiveproc!(stomach, args, state, {
    unpack_to_string!(args => name);
    let begin_mark = s!("\\begin{{{}}}", name);
    let end_mark = s!("\\end{{{}}}", name);
    DefConstructorI!(T_CS!(begin_mark), None, None, state,
    after_digest => sub!(move |stomach: &mut Stomach, whatsit: &mut Whatsit, _state: &mut State| {
      let mut nlines = 0;
      let gullet = &mut stomach.gullet;
      gullet.read_raw_line();    // IGNORE 1st line (after the \begin{$name} !!!
      while let Some(line) = gullet.read_raw_line() {
        if line == end_mark {
          break;
        }
        nlines += 1;
      }
      note_progress(&s!("[Skipped {} ({} lines)]",name,nlines));
      Ok(Vec::new())
    }));
  });

  // I don't understand Rust closures enough to figure out how to clone one, so instantiating it
  // twice instead, via a macro
  macro_rules! define_included {
    () => {
      primitiveproc!(stomach, args, inner_state, {
        args.reverse(); // we'll be using .pop() from the front
        let name = args.pop().unwrap_or(Tokens!()).to_string();
        let mut before_tokens = args.pop().unwrap_or(Tokens!()).unlist();
        before_tokens.push(T_CS!("\\ignorespaces"));
        let mut after_tokens = args.pop().unwrap_or(Tokens!()).unlist();
        after_tokens.push(T_CS!("\\ignorespaces"));
        // Note that we define the `magic' environment control sequences,
        // but DO NOT do any of the normal environ things, like \begingroup \endgroup!
        DefMacroI!(T_CS!(s!("\\begin{{{}}}", name)),
          None,
          sub[gullet, _args, _inner_state] {
            gullet.read_raw_line(); // IGNORE 1st line (after the \begin{$name} !!!
            Ok(before_tokens.clone().into())
          },
          inner_state);
        DefMacroI!(T_CS!(s!("\\end{{{}}}", name)),
          None,
          Tokens::new(after_tokens),
          inner_state
        );
      });
    };
  }

  let mut mock_stomach = Stomach::default();
  define_excluded(&mut mock_stomach, vec![Tokenize!("comment")], outer_state)?;

  DefPrimitiveI!("\\includecomment{}", define_included!());
  DefPrimitiveI!("\\excludecomment{}", define_excluded);
  DefPrimitiveI!("\\specialcomment{}{}{}", define_included!());
  DefPrimitiveI!("\\processcomment{}{}{}{}", noprimitive!());

  Ok(())
}
