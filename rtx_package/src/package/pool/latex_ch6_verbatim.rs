use crate::package::*;

lazy_static! {
  static ref END_VERBATIM_RE: Regex = Regex::new(r"^(.*?)\\end\{verbatim\}(.*?)$").unwrap();
  static ref T_OTHER_STAR : Token = T_OTHER!("*");
  static ref SEMIVERBATIM_CHARS : Vec<char> = vec!['%', '\\', '{', '}'];
}

//**********************************************************************
// C.6.4 Verbatim
//**********************************************************************
LoadDefinitions!(outer_state, {
  // NOTE: how's the best way to get verbatim material through?
  DefEnvironment!("{verbatim}", "<ltx:verbatim>#body</ltx:verbatim>");
  DefEnvironment!("{verbatim*}", "<ltx:verbatim>#body</ltx:verbatim>");

  DefMacro!("\\@verbatim", r"\par\aftergroup\lx@end@verbatim\lx@@verbatim"); // Close enough?
  DefConstructor!("\\lx@@verbatim", "<ltx:verbatim font='#font'>",
  before_digest => sub[stomach,state] {
    state.begin_semiverbatim(Some(&SEMIVERBATIM_CHARS));
    merge_font(fontmap!(family => "typewriter", series => "medium", shape => "upright"), state);
    state.assign_catcode(' ', Catcode::ACTIVE, None);  // Do NOT (necessarily) skip spaces after \verb!!!
    Let!(&T_ACTIVE!(" "), T_SPACE!());
  });
  DefConstructor!(r"\lx@end@verbatim", "</ltx:verbatim>",
    before_digest => sub[stomach,state] { state.end_semiverbatim()?; });

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
      // makes you wonder if the `get_font` API should be working with Arc<Font> in the first place...
      let font : Option<Arc<Font>> = whatsit.get_font().map(|ft| Arc::new((*ft).to_owned()));
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
  // And clearly separate expansion from digestion
  DefMacro!("\\verb", sub[gullet, args, state] {
    let active_chars = vec!['%', '\\', '{', '}'];
    state.begin_semiverbatim(Some(&active_chars));
    state.assign_catcode(' ', Catcode::ACTIVE, None); // Do NOT (necessarily) skip spaces after \verb!!!
    let mut init = gullet.read_token(state);
    let mut starred = false;
    if let Some(ref init_token) = init {
      if init_token == &(*T_OTHER_STAR) {
        starred = true;
        init = gullet.read_token(state);
      }
    }
    if let Some(init_token) = init {
      let init_str = init_token.get_string();
      let init_ch = init_str.chars().next().unwrap();
      state.assign_catcode(init_ch, Catcode::ACTIVE, None);
      let delim = Tokens!(T_ACTIVE!(init_ch));
      let body = gullet.read_until(&[&delim], state)?;
      state.end_semiverbatim()?;

      let mut result = vec![T_CS!("\\@hidden@bgroup")];
      if starred {
        result.push(T_CS!("\\lx@use@visiblespace"));
      }
      let mut inv_args = Vec::new();
      if starred {
        inv_args.push(Tokens!(T_OTHER!("*")));
      } else {
        inv_args.push(Tokens!());
      }
      inv_args.push(Tokens!(init_token));
      inv_args.push(body);
      result.extend(Invocation!(T_CS!("\\@internal@verb"), inv_args, gullet, state)?.unlist());
      result.push(T_CS!("\\@hidden@egroup"));
      Ok(Tokens!(result))
    } else { // typically something read too far got \verb and the content is somewhere else..?
      Error!("expected", "delimiter", gullet, state, "Verbatim argument lost\n Bindings for preceding code is probably broken");
      state.end_semiverbatim()?;
      Ok(Tokens!())
    }
  });

  DefPrimitive!("\\lx@use@visiblespace", sub[stomach, args, state] {
    state.assign_catcode(' ', Catcode::ACTIVE, None); // Do NOT (necessarily) skip spaces after \verb!!!
    Let!(&T_ACTIVE!(" "), T_OTHER!("\u{2423}")); // Visible space
  });

  DefConstructor!("\\@internal@verb{} Undigested {}",
    "?#isMath(<ltx:XMTok font='#font'>#text</ltx:XMTok>)(<ltx:verbatim font='#font'>#text</ltx:verbatim>)",
    properties => sub[stomach, args, state] {
      unpack!(args => a1, a2, a3);
      Ok(map!("text" => Stored::String(a3.to_string())))
    },
  font => { family => "typewriter", series => "medium", shape => "upright" },
  before_construct => sub[doc, whatsit, state] {
    if !whatsit.is_math() && !doc.can_contain(&doc.get_element().unwrap(), "#PCDATA", state) {
      doc.open_element("ltx:p", None, None, state)?;
    }
  },
  reversion => "\\verb#1#2#3#2");


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
    AssignValue!("PRESERVE_NEWLINES", 1);
  });
  DefPrimitive!("\\restorecr", {
    AssignValue!("PRESERVE_NEWLINES", 0);
  });
  DefMacro!(T_CS!("\\normalsfcodes"), None, Tokens!());
});
