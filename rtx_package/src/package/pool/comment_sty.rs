use crate::package::*;

LoadDefinitions!(outer_state, {
  //**********************************************************************
  // Define \name and \begin{name} to start an ignored section
  // until \endname or \end{name}, respectively
  let define_excluded: PrimitiveClosure = Rc::new(primitiveproc!(stomach, args, state, {
    unpack_to_string!(args => name);
    let begin_mark = s!("\\begin{{{}}}", name);
    let end_mark = s!("\\end{{{}}}", name);
    DefConstructor!(T_CS!(begin_mark), None, None,
    after_digest => sub[stomach, whatsit, after_digest_state] {
      let mut nlines = 0;
      let gullet = &mut stomach.gullet;
      gullet.read_raw_line(after_digest_state);    // IGNORE 1st line (after the \begin{$name} !!!
      while let Some(line) = gullet.read_raw_line(after_digest_state) {
        if line == end_mark {
          break;
        }
        nlines += 1;
      }
      note_progress(&s!("[Skipped {} ({} lines)]",name,nlines));
      Ok(Vec::new())
    });
  }));

  // I don't understand Rust closures enough to figure out how to clone one, so instantiating it
  // twice instead, via a macro
  let define_included: PrimitiveClosure = Rc::new(primitiveproc!(stomach, args, inner_state, {
    args.reverse(); // we'll be using .pop() from the front
    let name = args.pop().unwrap_or(Tokens!()).to_string();
    let mut before_tokens = args.pop().unwrap_or(Tokens!()).unlist();
    before_tokens.push(T_CS!("\\ignorespaces"));
    let mut after_tokens = args.pop().unwrap_or(Tokens!()).unlist();
    after_tokens.push(T_CS!("\\ignorespaces"));
    // Note that we define the `magic' environment control sequences,
    // but DO NOT do any of the normal environ things, like \begingroup \endgroup!
    DefMacro!(T_CS!(s!("\\begin{{{}}}", name)),
    None,
    sub[gullet, _args, macro_state] {
      gullet.read_raw_line(macro_state); // IGNORE 1st line (after the \begin{$name} !!!
      Ok(before_tokens.clone().into())
    });
    DefMacro!(T_CS!(s!("\\end{{{}}}", name)), None, Tokens::new(after_tokens));
  }));

  let mut mock_stomach = Stomach::default();
  define_excluded(&mut mock_stomach, vec![Tokenize!("comment", None)], outer_state)?;

  DefPrimitive!("\\includecomment{}", Rc::clone(&define_included));
  DefPrimitive!("\\excludecomment{}", define_excluded);
  DefPrimitive!("\\specialcomment{}{}{}", define_included);
  DefPrimitive!("\\processcomment{}{}{}{}", None);
});
