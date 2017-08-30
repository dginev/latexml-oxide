use package::*;

 pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  //**********************************************************************
  // Define \name and \begin{name} to start an ignored section
  // until \endname or \end{name}, respectively
  let define_excluded = primitivesub!(stomach, args, state,{
    let name = args[0].to_string();
    let begin_mark = format!("\\begin{{{}}}",name);
    let end_mark = format!("\\end{{{}}}", name);
    {
      DefConstructorI!(T_CS!(begin_mark), None, noreplacement!(),
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
          note_progress(&format!("[Skipped {} ({} lines)]",name,nlines));
          Ok(Vec::new())
        })
      ,state);
    }
    Ok(Vec::new())
  });

  // I don't understand Rust closures enough to figure out how to clone one, so instantiating it twice instead, via a macro
  macro_rules! define_included {() =>(primitivesub!(stomach, args, state,{
    args.reverse(); // we'll be popping from the front
    let name = if let Some(name_token) = args.pop() {
      name_token.to_string()
    } else {
      String::new()
    };
    // TODO: All instances of `.clone` here look like Rust memory waste, consider improving
    let mut before_tokens = if let Some(before) = args.pop() {
      before.unlist()
    } else {
      Vec::new()
    };
    before_tokens.push(T_CS!("\\ignorespaces"));
    let mut after_tokens = if let Some(after) = args.pop() {
      after.unlist()
    } else {
      Vec::new()
    };
    after_tokens.push(T_CS!("\\ignorespaces"));
    // Note that we define the `magic' environment control sequences,
    // but DO NOT do any of the normal environ things, like \begingroup \endgroup!
    DefMacroI!(T_CS!(format!("\\begin{{{}}}",name)), None, move |gullet, _args, _state| {
        gullet.read_raw_line();    // IGNORE 1st line (after the \begin{$name} !!!
        Ok(Tokens::new(before_tokens.clone()))
      }, state);
    DefMacroI!(T_CS!(format!("\\end{{{}}}",name)), None, move |_gullet, _args, _state| {
      Ok(Tokens::new(after_tokens.clone()))
    }, state);

    Ok(Vec::new())
  });)}

  let mut mock_stomach = Stomach::default();
  try!(define_excluded(&mut mock_stomach, vec![Tokens{tokens: vec![T_OTHER!("comment")]}], state));

  DefPrimitiveI!("\\includecomment{}",     define_included!());
  DefPrimitiveI!("\\excludecomment{}",     define_excluded);
  DefPrimitiveI!("\\specialcomment{}{}{}", define_included!());
  DefPrimitiveI!("\\processcomment{}{}{}{}", noprimitive!());

  Ok(())
}
