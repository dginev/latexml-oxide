use crate::package::*;

lazy_static! {
  static ref END_VERBATIM_RE: Regex = Regex::new(r"^(.*?)\\end\{verbatim\}(.*?)$").unwrap();
}

//**********************************************************************
// C.6.4 Verbatim
//**********************************************************************
LoadDefinitions!(outer_state, {
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

  DefConstructor!(T_CS!("\\begin{verbatim}"), None, "<ltx:verbatim font='#font'>#body</ltx:verbatim>",
    before_digest => sub[stomach, state] {
      stomach.bgroup(state);
      let mut stuff = Vec::new();
      if let Some(b) = state.lookup_tokens("@environment@verbatim@atbegin") {
        stuff.push(stomach.digest(b.unlist(), state)?);
      }
      AssignValue!("current_environment", "verbatim");
      DefMacro!("\\@currenvir", "verbatim");
      MergeFont!(family => "typewriter");
      Ok(stuff)
    },
    after_digest => sub[stomach, whatsit, state] {
      // makes you wonder if the `get_font` API should be working with Rc<Font> in the first place...
      let font : Option<Rc<Font>> = whatsit.get_font().map(|ft| Rc::new((*ft).to_owned()));
      let loc = whatsit.get_locator();
      let mut lines : Vec<String> = Vec::new();
      let gullet = stomach.get_gullet_mut();
      while let Some(line) = gullet.read_raw_line(state) {
        // The raw chars will still have to be decoded (but not space!!)
        let decoded_line : String = line.chars()
          .map(|c| if c == ' ' {" ".to_string() } else {
             font::decode_string(&c.to_string(), Some("OT1_typewriter"), true, state) })
          .collect::<Vec<String>>().join("");
        if let Some(caps) = END_VERBATIM_RE.captures(&decoded_line) {
          let pre = s!("{}\n", caps.get(1).map_or("", |m| m.as_str()));
          let post = caps.get(2).map_or("", |m| m.as_str());
          lines.push(pre);
          let mut post_tokens = Tokenize!(post).unlist();
          post_tokens.push(T_CR!());
          gullet.unread(Tokens::new(post_tokens));
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
      let boxes = lines.into_iter().map(|line|
        Tbox::new(line.clone(), font.clone(), Some(loc.clone().into_owned()), T_OTHER!(line).into(), HashMap::new(), state).into()
      ).collect();
      whatsit.set_body(boxes);
    },
    before_construct => sub[document, whatsit, state] { document.maybe_close_element("ltx:p", state)?; }
  );

  DefPrimitive!("\\@vobeyspaces", sub[stomach, args, state] {
    AssignCatcode!(' ', Catcode::ACTIVE);
    Let!(&T_ACTIVE!(" "), T_CS!("\\nobreakspace"));
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
      Ok(Invocation!(cs, vec![Tokens!(init.unwrap()), body], gullet)?)
    } else { // typically something read too far got \verb and the content is somewhere else..?
      Error!("expected", "delimiter", gullet, state, "Verbatim argument lost\n Bindings for preceding code is probably broken");
      state.end_semiverbatim()?;
      Ok(Tokens!())
    }
  });

  DefConstructor!("\\@text@verb{}{}", "<ltx:verbatim font='#font'>#2</ltx:verbatim>",
    before_digest => sub[stomach, state] {
      stomach.bgroup(state);
      MergeFont!(family => "typewriter");
    },
    after_digest => sub[stomach,whatsit,state] { stomach.egroup(state)?; },
    // Since ltx:verbatim is both inline & block, we have to fudge inline mode
    before_construct => sub[document, args, state] {
      if !document.can_contain(&document.get_element().unwrap(), "#PCDATA", state) {
        document.open_element("ltx:p", None, None, state)?;
      }},
    reversion => "\\verb#1#2#1"
  );
  DefConstructor!("\\@math@verb{}{}", "#2",
   before_digest => sub[stomach, state] {
     stomach.bgroup(state);
     MergeFont!(family => "typewriter");
   },
   after_digest => sub[stomach,whatsit,state] { stomach.egroup(state)?; },
   reversion => "\\verb#1#2#1"
  );

  // Actually, latex sets catcode to 13 ... is this close enough?
  DefPrimitive!("\\obeycr", {
    AssignValue!("PRESERVE_NEWLINES", true);
  });
  DefPrimitive!("\\restorecr", {
    AssignValue!("PRESERVE_NEWLINES", false);
  });
  DefMacro!(T_CS!("\\normalsfcodes"), None, Tokens!());
});
