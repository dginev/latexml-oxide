use crate::package::*;

lazy_static! {
  static ref END_VERBATIM_RE: Regex = Regex::new(r"^(.*?)\\end\{verbatim\}(.*?)$").unwrap();
}

//**********************************************************************
// C.6.4 Verbatim
//**********************************************************************
pub fn load_definitions(outer_state: &mut State) -> Result<()> {
  SetupBindingMacros!(outer_state);

  // NOTE: how's the best way to get verbatim material through?
  DefEnvironment!("{verbatim}", "<ltx:verbatim>#body</ltx:verbatim>");
  DefEnvironment!("{verbatim*}", "<ltx:verbatim>#body</ltx:verbatim>");
  Let!("\\@verbatim", "\\verbatim");
  // Close enough?
  // verbatim is a bit of special case;
  // It looks like an environment, but it only ends with an explicit "\end{verbatim}" on it's own
  // line. So, we'll end up doing things more manually.
  // We're going to sidestep the Gullet for inputting,
  // and also the usual environment capture.

  DefConstructor!(cs["\\begin{verbatim}"], None, "<ltx:verbatim font='#font'>#body</ltx:verbatim>",
    before_digest => beforesub!(stomach, state, {
      stomach.bgroup(state);
      let mut stuff = Vec::new();
      if let Some(b) = state.lookup_tokens("@environment@verbatim@atbegin") {
        stuff.push(stomach.digest(b.unlist(), state)?);
      }
      state.assign_value("current_environment", "verbatim", None);
      DefMacro!("\\@currenvir", "verbatim", state);
      MergeFont!(family => "typewriter", state);
      Ok(stuff)
    }),
    after_digest => afterproc!(stomach, whatsit, state, {
      let font : Option<Rc<Font>> = match whatsit.get_font() {
        None => None,
        Some(ft) => Some(Rc::new((*ft).to_owned()))
      };
      let loc = whatsit.get_locator();
      let mut lines : Vec<String> = Vec::new();
      let gullet = stomach.get_gullet_mut();
      while let Some(line) = gullet.read_raw_line() {
        // The raw chars will still have to be decoded (but not space!!)
        let decoded_line : String = line.chars()
          .map(|c| if c == ' ' {" ".to_string() } else {
             font::decode_string(&c.to_string(), Some("OT1_typewriter"), true, state) })
          .collect::<Vec<String>>().join("");
        if let Some(caps) = END_VERBATIM_RE.captures(&decoded_line) {
          let pre = s!("{}\n", caps.get(1).map_or("", |m| m.as_str()));
          let post = caps.get(2).map_or("", |m| m.as_str());
          lines.push(pre);
          let mut post_tokens = Tokenize!(post, state).unlist();
          post_tokens.push(T_CR!());
          gullet.unread(&Tokens::new(post_tokens));
          break;
        } else {
          lines.push(s!("{}\n", line));
        }
      }
      if let Some(last_line) = lines.last() {
        if last_line == "\n" {
          lines.pop();
        }
      }
      // Note last line ends up as Whatsit's "trailer"
      if let Some(b) = state.lookup_tokens("@environment@verbatim@atend") {
        lines.push(stomach.digest(b, state)?.to_string());
      }
      stomach.egroup(state)?;
      lines.push("\\end{verbatim}".to_string());
      whatsit.set_body(lines.into_iter().map(|line|
        Tbox::new(line.clone(), font.clone(), loc.clone(), T_OTHER!(line).into(), HashMap::new(), state).into()
      ).collect());
    }),
    before_construct => construct!(document, whatsit, state, { document.maybe_close_element("ltx:p", state)?; })
  );

  DefPrimitiveI!("\\@vobeyspaces", |stomach, args, state| {
    state.assign_catcode(' ', Catcode::ACTIVE, None);
    LetI!(&T_ACTIVE!(" "), T_CS!("\\nobreakspace"), state);
    Ok(vec![])
  });

  // WARNING: Need to be careful about what catcodes are active here
  DefMacro!("\\verb", sub[gullet, args, state] {
    let mouth = gullet.get_mouth_mut().unwrap();
    state.begin_semiverbatim(Some(vec!['%', '\\', '{', '}']));
    let mut init = mouth.read_token(state);
    if let Some(ref init_token) = init {
      if init_token.as_str() == "*" {
        init = mouth.read_token(state); // Should I bother handling \verb* ?
      }
    }
    if let Some(ref init_token) = init {
      let body = mouth.read_tokens(Some(init_token), state);
      state.end_semiverbatim()?;
      let cs = if state.lookup_bool("IN_MATH") { T_CS!("\\@math@verb") } else { T_CS!("\\@text@verb") };
      Ok(Invocation!(cs, vec![Tokens!(init.unwrap()), body], gullet, state)?)
    } else { // typically something read too far got \verb and the content is somewhere else..?
      error!(target: "expected:delimiter", "Verbatim argument lost\n Bindings for preceding code is probably broken");
      state.end_semiverbatim()?;
      Ok(Tokens!())
    }
  });

  DefConstructor!("\\@text@verb{}{}", "<ltx:verbatim font='#font'>#2</ltx:verbatim>",
    before_digest => beforeproc!(stomach, state, {
      stomach.bgroup(state);
      MergeFont!(family => "typewriter", state);
    }),
    after_digest => afterproc!(stomach,whatsit,state, { stomach.egroup(state)?; }),
    // Since ltx:verbatim is both inline & block, we have to fudge inline mode
    before_construct => construct!(document, args, state, {
      if !document.can_contain(&document.get_element().unwrap(), "#PCDATA", state) {
        document.open_element("ltx:p", None, None, state)?;
      }}),
    reversion => "\\verb#1#2#1".into_option()
  );
  DefConstructor!("\\@math@verb{}{}", "#2",
   before_digest => beforeproc!(stomach, state, {
     stomach.bgroup(state);
     MergeFont!(family => "typewriter", state);
   }),
   after_digest => afterproc!(stomach,whatsit,state, { stomach.egroup(state)?; }),
   reversion => "\\verb#1#2#1".into_option()
  );

  // Actually, latex sets catcode to 13 ... is this close enough?
  DefPrimitiveI!("\\obeycr", |stomach, whatsit, state| {
    state.assign_value("PRESERVE_NEWLINES", true, None);
    Ok(vec![])
  });
  DefPrimitiveI!("\\restorecr", |stomach, whatsit, state| {
    state.assign_value("PRESERVE_NEWLINES", false, None);
    Ok(vec![])
  });
  DefMacroI!(T_CS!("\\normalsfcodes"), None, Tokens!());

  Ok(())
}
