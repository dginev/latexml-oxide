use package::*;

pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  //**********************************************************************
  // Define \name and \begin{name} to start an ignored section
  // until \endname or \end{name}, respectively
  let define_excluded = primitivesub!(stomach, args, state,{
    let name = args[0].to_string();
    let endmark = format!("\\end{{{}}}", name);
    {
      DefConstructorI_F!(T_CS!(format!("\\begin{{{}}}",name)), None, noreplacement!()
        // after_Digest => [sub {
        //     my ($istomach, $whatsit) = @_;
        //     let nlines = 0;
        //     my ($line);
        //     let gullet = $istomach->getGullet;
        //     $gullet->readRawLine;    // IGNORE 1st line (after the \begin{$name} !!!
        //     while (defined($line = $gullet->readRawLine) && ($line ne $endmark)) {
        //       $nlines++; }
        //     NoteProgress("[Skipped $name ($nlines lines)]");
        // }]
      ,state);
    }
    Ok(Vec::new())
  });

  let define_included = primitivesub!(stomach, args, state,{
    // let name = args[0].to_string();
    // let before = args[1];
    // let after = args[2];
    // let mut before_tokens = before.unlist();
    // before_tokens.push(T_CS!("\\ignorespaces"));
    // let mut after_tokens = after.unlist();
    // after_tokens.push(T_CS!("\\ignorespaces"));
    // Note that we define the `magic' environment control sequences,
    // but DO NOT do any of the normal environ things, like \begingroup \endgroup!
    // DefMacroI!(T_CS!(format!("\\begin{{{}}}",name)), None, move |gullet, _args, _state| {
    //     gullet.read_raw_line();    // IGNORE 1st line (after the \begin{$name} !!!
    //     Ok(before_tokens)
    //   });
    // DefMacroI!(T_CS!(format!("\\end{{{}}}",name)), None, move |_gullet, _args, _state| {
    //   Ok(after_tokens)
    // });

    Ok(Vec::new())
  });

  let define_special_included = primitivesub!(stomach, args, state,{
    // let name = args[0].to_string();
    // let before = args[1];
    // let after = args[2];
    // let mut before_tokens = before.unlist();
    // before_tokens.push(T_CS!("\\ignorespaces"));
    // let mut after_tokens = after.unlist();
    // after_tokens.push(T_CS!("\\ignorespaces"));
    // Note that we define the `magic' environment control sequences,
    // but DO NOT do any of the normal environ things, like \begingroup \endgroup!
    // DefMacroI!(T_CS!(format!("\\begin{{{}}}",name)), None, move |gullet, _args, _state| {
    //     gullet.read_raw_line();    // IGNORE 1st line (after the \begin{$name} !!!
    //     Ok(before_tokens)
    //   });
    // DefMacroI!(T_CS!(format!("\\end{{{}}}",name)), None, move |_gullet, _args, _state| {
    //   Ok(after_tokens)
    // });

    Ok(Vec::new())
  });


  let mut mock_stomach = Stomach::default();
  define_excluded(&mut mock_stomach, vec![Tokens{tokens: vec![T_OTHER!("comment")]}], state);

  DefPrimitiveI!("\\includecomment{}",     define_included);
  DefPrimitiveI!("\\excludecomment{}",     define_excluded);
  DefPrimitiveI!("\\specialcomment{}{}{}", define_special_included);
  //DefPrimitive("\processcomment{}{}{}{}",);

  Ok(())
}
