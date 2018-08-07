use package::*;

// Hmm... I wonder, should getString itself be dealing with escapechar?
fn escapechar(state: &State) -> String {
  let code = match state.lookup_register("\\escapechar", Vec::new()) {
    Some(RegisterValue::Number(v)) => v.value_of(),
    _ => -1,
  };
  if code >= 0 && code <= 255 {
    let char_code = (code as u8) as char;
    char_code.to_string()
  } else {
    String::new()
  }
}

pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  DefConditional!("\\ifx Token Token", sub[gullet, args, inner_state] {
    unpack!(args => token1, token2);
    let token1 : Token = token1.into();
    let token2 : Token = token2.into();
    let xequals = XEquals!(&token1, &token2, inner_state);
    Ok(xequals)
  });


// TODO:
// DefConditionalI('\iftrue',  undef, sub { 1; });
// DefConditionalI('\iffalse', undef, sub { 0; });

//======================================================================
// This makes \relax disappear completely after digestion
// (which seems most TeX like).
DefPrimitiveI!("\\relax", noprimitive!() );
//// However, this keeps a box, so it can appear in UnTeX
////// DefPrimitive('\relax',undef);
//// But if you do that, you've got to watch out since it usually
//// shouldn't be a box; See the isRelax code in handleScripts, below

// TODO:
// DefMacro('\number Number', sub { Explode($_[1]->valueOf); });

// define it here (only approxmiately), since it's already useful.
Let!("\\protect", "\\relax");

//======================================================================


  DefParameterType!("CSName", reader => Rc::new(|gullet: &mut Gullet, _inner: Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {
    let mut cs = escapechar(state);
    // keep newlines from having \n inside!
    while let Some(token) = gullet.read_x_token(true, true, state)? {
      let s = token.get_string();
      if s == "\\endcsname" {
        break;
      }
      let cc = token.get_catcode();
      if cc == Catcode::CS {
        if let Some(defn) = state.lookup_definition(&token) {
          error!(target: &s!("unexpected:{}", token), "The control sequence {:?} should not appear between \\csname and \\endcsname", token);
        } else {
          error!(target: &s!("undefined:{}", token), "The token {:?} is not defined", token);
        } 
      } else if cc == Catcode::SPACE {  // Keep newlines from having \n!
        cs.push(' ');
      } else { 
        cs.push_str(s);
      }
    }
    T_CS!(cs).into()
  }));

  DefMacro!("\\csname CSName", sub[gullet, args, state] {
    unpack!(args => token);
    let token : Token = token.into();
    if state.lookup_meaning(&token).is_none() { 
      Let!(token, "\\relax", state);
    }
    token.into()
  });

  DefPrimitive!("\\endcsname", sub[stomach, whatsit, state] {
    error!(target: "unexpected:\\endcsname", "Extra \\endcsname");
    Ok(Vec::new())
  });

  DefMacro!("\\expandafter Token Token", sub[gullet, args, state] {
    unpack!(args => tok, xtok);
    let mut tokens : Vec<Token> = vec![tok.into()];
    let xtok_single = xtok.clone().into();
    if let Some(defn) = state.lookup_expandable(&xtok_single, false) {
      // Note that IF expandafter ends up expanding a \the in an \edef,
      // that it Overrides the implicit noexpand that \edef would normally use for\the!!
      state.remove_value("NOEXPAND_THE");
      tokens.append(&mut defn.invoke(gullet, state)?.unlist()); // Expand $xtok ONCE ONLY!
    } else {
      tokens.append(&mut xtok.unlist()); 
    };
    Ok(tokens.into())
  });

  // The following special cases are built-in to Definition
  DefConditional!("\\else",          None);
  DefConditional!("\\or",            None);
  DefConditional!("\\fi",            None);
  DefConditional!("\\ifcase Number", None);

  Ok(())
}
