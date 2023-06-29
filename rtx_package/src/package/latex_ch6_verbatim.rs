use crate::package::*;

static END_VERBATIM_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(.*?)\\end\{verbatim\}(.*?)$").unwrap());
static SEMIVERBATIM_CHARS: [char;4] = ['%', '\\', '{', '}'];
static T_OTHER_STAR: Lazy<Token> = Lazy::new(|| T_OTHER!("*"));

//**********************************************************************
// C.6.4 Verbatim
//**********************************************************************
#[rustfmt::skip]
LoadDefinitions!({
  // NOTE: how's the best way to get verbatim material through?
  DefEnvironment!("{verbatim}", "<ltx:verbatim>#body</ltx:verbatim>");
  DefEnvironment!("{verbatim*}", "<ltx:verbatim>#body</ltx:verbatim>");

  DefMacro!(
    "\\@verbatim",
    r"\par\aftergroup\lx@end@verbatim\lx@@verbatim"
  ); // Close enough?
  DefConstructor!("\\lx@@verbatim", "<ltx:verbatim font='#font'>",
  before_digest => {
   begin_semiverbatim(Some(&SEMIVERBATIM_CHARS));
    merge_font(fontmap!(family => "typewriter", series => "medium", shape => "upright"));
    assign_catcode(' ', Catcode::ACTIVE, None);  // Do NOT (necessarily) skip spaces after \verb!!!
    Let!(&T_ACTIVE!(' '), T_SPACE!());
  });
  DefConstructor!(r"\lx@end@verbatim", "</ltx:verbatim>",
    before_digest => { end_semiverbatim()?; });

  // Close enough?
  // verbatim is a bit of special case;
  // It looks like an environment, but it only ends with an explicit "\end{verbatim}" on it's own
  // line. So, we'll end up doing things more manually.
  // We're going to sidestep the Gullet for inputting,
  // and also the usual environment capture.

  DefConstructor!(T_CS!("\\begin{verbatim}"), None, "<ltx:verbatim font='#font'>#body</ltx:verbatim>",
    before_digest => {
      bgroup();
      let mut stuff = Vec::new();
      if let Some(b) = state::lookup_tokens("@environment@verbatim@atbegin") {
        stuff.push(stomach::digest(b.unlist())?);
      }
      AssignValue!("current_environment", "verbatim");
      DefMacro!("\\@currenvir", "verbatim");
      MergeFont!(family => "typewriter");
      Ok(stuff)
    },
    after_digest => sub[whatsit] {
      // makes you wonder if the `get_font` API should be working with Rc<Font> in the first place...
      let font : Option<Rc<Font>> = whatsit.get_font()?.map(|ft| Rc::new((*ft).to_owned()));
      let loc = whatsit.get_locator();
      let mut lines : Vec<SymbolU32> = Vec::new();
      while let Some(line) = gullet::read_raw_line() {
        // The raw chars will still have to be decoded (but not space!!)
        let mut decoded_line : String = String::new();
        for c in line.chars() {
          if c == ' ' { decoded_line.push(' '); }
          else {
            let decoded_c = font::decode_string(arena::pin_char(c), Some("OT1_typewriter"), true);
            arena::with(decoded_c, |c_str| decoded_line.push_str(c_str));
          }
        }
        if let Some(caps) = END_VERBATIM_RE.captures(&decoded_line) {
          let pre = s!("{}\n", caps.get(1).map_or("", |m| m.as_str()));
          let post = caps.get(2).map_or("", |m| m.as_str());
          lines.push(arena::pin(pre));
          let mut post_tokens = Tokenize!(post).unlist();
          post_tokens.push(T_CR!());
          gullet::unread(Tokens::new(post_tokens));
          break;
        } else {
          lines.push(arena::pin(s!("{}\n", line)));
        }
      }
      if let Some(last_line) = lines.last() {
        if *last_line == arena::pin_static("\n") {
          lines.pop();
        }
      }
      // Note last line ends up as Whatsit's "trailer"
      if let Some(b) = state::lookup_tokens("@environment@verbatim@atend") {
        lines.push(arena::pin(stomach::digest(b)?.to_string()));
      }
      egroup()?;
      lines.push(arena::pin_static("\\end{verbatim}"));
      let boxes = lines.into_iter().map(|line|
        Tbox::new(line, font.clone(), loc.clone()
          .map(|l| l.into_owned()), Token{text: line, code:Catcode::OTHER, smuggled:None}.into(), HashMap::default()).into()
      ).collect();
      whatsit.set_body(boxes);
    },
    before_construct => sub[document, _whatsit] {
      document.maybe_close_element("ltx:p")?; }
  );

  DefPrimitive!("\\@vobeyspaces", {
    AssignCatcode!(' ', Catcode::ACTIVE);
    Let!(&T_ACTIVE!(' '), T_CS!("\\nobreakspace"));
  });
  DefMacro!("\\@xobeysp", "\\nobreakspace");

  // WARNING: Need to be careful about what catcodes are active here
  // And clearly separate expansion from digestion
  DefMacro!("\\verb", {
   begin_semiverbatim(Some(&SEMIVERBATIM_CHARS));
    // Do NOT (necessarily) skip spaces after \verb!!!
    assign_catcode(' ', Catcode::ACTIVE, None);
    let mut init = None;
    // As of texlive 2021, DO skip spaces before delimiter (even tho we've changed catcodes)
    let space_sym = arena::pin_static(" ");
    while let Some(maybe_init) = gullet::read_token()? {
      if maybe_init.get_sym() != space_sym {
        init = Some(maybe_init);
        break;
      }
    }
    let mut starred = false;
    if let Some(ref init_token) = init {
      if *init_token == *T_OTHER_STAR {
        starred = true;
        while let Some(maybe_init) =  gullet::read_token()? {
          if maybe_init.get_sym() != space_sym {
            init = Some(maybe_init);
            break;
          }
        }
      }
    }
    if let Some(init_token) = init {
      let init_ch = init_token.with_str(|is| is.chars().next().unwrap());
      assign_catcode(init_ch, Catcode::ACTIVE, None);
      let delim = Tokens!(T_ACTIVE!(init_ch));
      let body = gullet::read_until(&delim)?;
      end_semiverbatim()?;

      let mut result = vec![T_CS!("\\@hidden@bgroup")];
      if starred {
        result.push(T_CS!("\\lx@use@visiblespace"));
      }
      result.extend(Invocation!(T_CS!("\\@internal@verb"), vec![
        if starred { Tokens!(T_OTHER!("*")) } else { Tokens!() },
        Tokens!(init_token),
        body
      ]).unlist());
      result.push(T_CS!("\\@hidden@egroup"));
      Ok(Tokens::new(result))
    } else { // typically something read too far got \verb and the content is somewhere else..?
      Error!("expected", "delimiter",
        "Verbatim argument lost\n Bindings for preceding code is probably broken");
      end_semiverbatim()?;
      Ok(Tokens!())
    }
  });

  DefPrimitive!("\\lx@use@visiblespace", {
    // Do NOT (necessarily) skip spaces after \verb!!!
    assign_catcode(' ', Catcode::ACTIVE, None);
    // Visible space
    Let!(&T_ACTIVE!(' '), T_OTHER!("\u{2423}"));
  });

  // Arrange to digest the body in text mode, to keep (eg) "_" from turning to "\_"
  DefMacro!("\\@internal@verb{}{}{}",
      r"\ifmmode\@internal@math@verb{#1}{#2}{#3}\else\@internal@text@verb{#1}{#2}{#3}\fi");
  DefConstructor!("\\@internal@math@verb{} Undigested {}",
    "<ltx:XMTok font='#font'>#3</ltx:XMTok>",
    mode      => "text",
    font      => { family => "typewriter", series => "medium", shape => "upright" },
    reversion => "\\verb#1#2#3#2");
  DefConstructor!("\\@internal@text@verb{} Undigested {}",
    "<ltx:verbatim font='#font'>#3</ltx:verbatim>",
    font            => { family => "typewriter", series => "medium", shape => "upright" },
    before_construct => sub[doc,_whatsit] {
      if !document::can_contain(doc.get_element().as_ref().unwrap(), "#PCDATA") {
        doc.open_element("ltx:p", None, None)?;
      }
    },
    reversion => "\\verb#1#2#3#2");


  // Actually, latex sets catcode to 13 ... is this close enough?
  DefPrimitive!("\\obeycr", {
    AssignValue!("PRESERVE_NEWLINES", 1);
  });
  DefPrimitive!("\\restorecr", {
    AssignValue!("PRESERVE_NEWLINES", 0);
  });
  DefMacro!(T_CS!("\\normalsfcodes"), None, Tokens!());
});
