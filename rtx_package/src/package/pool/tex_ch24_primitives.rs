use crate::package::*;
use rtx_core::list::List;
use rtx_core::tbox::Tbox;
use rtx_core::TexMode;

//**********************************************************************
// Primitives
// See The TeXBook, Chapter 24, Summary of Vertical Mode
//  and Chapter 25, Summary of Horizontal Mode.
// Parsing of basic types (pp.268--271) is (mostly) handled in Gullet.pm
//**********************************************************************

LoadDefinitions!(state, {
  //======================================================================
  // Remaining Mode independent primitives in Ch.24, pp.279-280
  // \relax was done as expandable (isn't that right?)
  // }
  // Note, we don't bother making sure begingroup is ended by endgroup.

  // These define the handler for { } (or anything of catcode BEGIN, END)

  // These are actually TeX primitives, but we treat them as a Whatsit so they
  // remain in the constructed tree.
  DefPrimitive!("{", sub[stomach, (), state] {
    stomach.bgroup(state);
    let open = Tbox::new(String::new(), None, None, Tokens!(T_BEGIN!()), HashMap::new(), state);
    let mode = if LookupBool!("IN_MATH") {
      Some(TexMode::Math)
    } else {
      Some(TexMode::Text)
    };
    let body = stomach.digest_next_body(None, state)?;
    let mut boxes = vec![Digested::from(open)];
    boxes.extend(body);
    let mut font = None;
    for abox in boxes.iter().rev() {
      if let Some(boxfont) = abox.get_font(state)? {
        font = Some(boxfont.into_owned());
        break;
      }
    }
    List {
      boxes,
      mode,
      font,
      locator: Locator::default(),
      properties: HashMap::new()
    }
  });

  DefPrimitive!(
    "}",
    sub[stomach, (), state] {
      let f = LookupFont!();
      stomach.egroup(state)?;
      Tbox::new(String::new(), f, None, Tokens!(T_END!()), HashMap::new(), state)
    }
  );

  // // These are for those screwy cases where you need to create a group like box,
  // // more than just bgroup, egroup,
  // // BUT you DON'T want extra {, } showing up in any untex-ing.
  DefConstructor!("\\@hidden@bgroup", "#body",
    before_digest => sub[stomach,state] { stomach.bgroup(state); },
    capture_body => true,
    reversion=>"" //TODO
    // reversion => sub[whatsit, state] {
    //   whatsit.get_body().revert()
    // }
  );
  DefConstructor!("\\@hidden@egroup", "",
    after_digest => sub[stomach,args,state] { stomach.egroup(state)?; },
    reversion => ""
  );

  DefPrimitive!(
  "\\begingroup", sub[stomach, (), state] {
    stomach.begingroup(state);
  });
  DefPrimitive!(
  "\\endgroup", sub[stomach, (), state] {
    stomach.endgroup(state)?;
  });

  // // Debugging aids; Ignored!
  DefPrimitive!("\\show Token", None);
  DefPrimitive!("\\showbox Number", None);
  DefPrimitive!("\\showlists", None);
  DefPrimitive!("\\showthe Token", None);

  // DefPrimitive('\shipout ??
  DefPrimitive!("\\ignorespaces SkipSpaces", None);

  DefPrimitive!("\\lx@ignorehardspaces", sub[stomach, (), state] {
    let mut boxes = Vec::new();
    while let Some(token) = stomach.get_gullet_mut().read_x_token(None, false, state)? {
      boxes = stomach.invoke_token(&token, state)?;
      if boxes.is_empty() {
        break;
      }
      while !boxes.is_empty() {
        let is_space = if let Some(space_val) = boxes[0].get_property("isSpace") {
          match space_val {
            Cow::Borrowed(Stored::Bool(space_bool)) => *space_bool,
            Cow::Owned(Stored::Bool(ref space_bool))  => *space_bool, // TODO : is there match syntax for Cow ?
            _ => false
          }
        } else {
          false
        };

        if is_space {
          boxes = boxes[1..].to_vec();
        } else {
          break;
        }
      }

      if !boxes.is_empty() {
        break;
      }
    }
    Ok(boxes)
  });

  // \afterassignment saves ONE token (globally!) to execute after the next assignment
  DefPrimitive!("\\afterassignment Token", sub[stomach, (t), state] {
    state.assign_value("afterAssignment", t, Some(Scope::Global));
  });
  // \aftergroup saves ALL tokens (from repeated calls) to be executed IN ORDER after the next egroup or }
  DefPrimitive!("\\aftergroup Token", sub[stomach, (t), state] {
    state.push_value("afterGroup", t);
  });

  // \uppercase<general text>, \lowercase<general text>

  // Note that these are NOT expandable, even though the "return" tokens!
  DefPrimitive!("\\uppercase GeneralText", sub[stomach,(tokens), state] {
    let result = tokens.unlist().into_iter()
    .map(|t| uppercase_token(t, state))
    .collect::<Vec<Token>>();
    stomach.get_gullet_mut().unread(Tokens::new(result));
  });
  DefPrimitive!("\\lowercase GeneralText", sub[stomach,(tokens), state] {
    let result = tokens.unlist().into_iter()
    .map(|t| lowercase_token(t, state))
    .collect::<Vec<Token>>();
    stomach.get_gullet_mut().unread(Tokens::new(result));
  });

  DefPrimitive!("\\message{}", sub [stomach, (message), state] {
    if state.lookup_int("VERBOSITY") > -1 {
      eprintln!("{}", writable_tokens(
        do_expand(message, stomach.get_gullet_mut(), state)?, state)?);
    }
  });

  // DefRegister('\errhelp' => Tokens());
  // DefPrimitive('\errmessage{}', sub {
  //     my ($stomach, $stuff) = @_;
  // print STDERR ToString(Expand($stuff)) . ": " . ToString(Expand(Tokens(T_CS('\the'),
  // T_CS('\errhelp')))) . "\n";     return; });

  // TeX I/O primitives
  DefPrimitive!("\\openin Number SkipMatch:= SkipSpaces TeXFileName",
  sub[stomach, (port, filename), state] {
    let port = port.to_string();
    let filename = filename.to_string();
    // possibly should close $port if it's already been opened?
    // Rely on FindFile to enforce any access restrictions
    if let Some(path) = find_file(&filename, Some(FindFileOptions {forbid_ltxml: true, ..FindFileOptions::default()}), state) {
      let content_str = LookupString!(&s!("{}_contents",path));
      let content = if content_str.is_empty() {
        None
      } else {
        Some(content_str)
      };
      let mouth = Mouth::create(&path, MouthOptions {
        content,
        .. MouthOptions::default()
      }, state)?;
      AssignValue!(&s!("input_file:{}", port), mouth, Some(Scope::Global));
    }
  });

  DefPrimitive!("\\closein Number", sub[stomach, (port), state] {
    // Clone the Rc<> for mouth out of state, since we'll be mutating.
    let mouth_opt = if let Some(Stored::Mouth(ref mouth)) = LookupValue!(&s!("input_file:{}", port)) {
      Some(Arc::clone(mouth))
    } else {
      None
    };
    //   close the mouth (if any) and clear the variable
    if let Some(mouth) = mouth_opt {
      mouth.write().unwrap().finish(state);
      AssignValue!(&s!("input_file:{}", port), false, Some(Scope::Global));
    }
  });

  DefPrimitive!("\\read Number SkipKeyword:to SkipSpaces Token", sub[stomach, (port, token), state] {
    if let Some(Stored::Mouth(mouth_stored)) = state.lookup_value(&format!("input_file:{port}")) {
      let mouth_obj = Arc::clone(mouth_stored);
      stomach.bgroup(state);
      AssignValue!("PRESERVE_NEWLINES", 2); // Special EOL/EOF treatment for \read
      AssignValue!("INCLUDE_COMMENTS", false);
      let mut tokens = Vec::new();
      let mut level = 0;
      let mut mouth = mouth_obj.write().unwrap();
      while let Some(t) = mouth.read_token(state) {
        let cc = t.get_catcode();
        if cc != Catcode::MARKER {
          tokens.push(t);
        }
        match cc {
          Catcode::BEGIN => {level += 1},
          Catcode::END => {level -= 1},
          _ => {}
        };
        if level == 0 && mouth.is_eol(state) {
          break;
        }
      }
      stomach.egroup(state)?;
      DefMacro!(token, None, Tokens::new(tokens), nopack_parameters => true);
    }
  });

  DefConditional!("\\ifeof Number", sub[gullet, (port), state] {
    if let Some(Stored::Mouth(mouth)) = LookupValue!(&s!("input_file:{}", port)) {
      mouth.read().unwrap().at_eof()
    } else {
      true
    }
  });

  // For output files, we'll write the data to a cached internal copy
  // rather than to the actual file system.
  DefPrimitive!("\\openout Number SkipMatch:= SkipSpaces TeXFileName", sub[stomach, (port, filename), state] {
    let port = port.to_string();
    let filename = filename.to_string();
    let contents_key = &s!("{}_contents",filename);
    AssignValue!(&s!("output_file:{}",port)  => filename,  Some(Scope::Global));
    AssignValue!(contents_key => "",  Some(Scope::Global));
  });

  DefPrimitive!("\\closeout Number", sub[stomach, (port), state] {
    AssignValue!(&s!("output_file:{}",port), false, Some(Scope::Global));
  });

  DefPrimitive!("\\write Number {}", sub[stomach, (port, tokens), state] {
    if let Some(filename) = LookupValue!(&s!("output_file:{}", port)) {
      let handle   = s!("{}_contents",filename);
      let mut contents : String = LookupString!(&handle);
      let mut gullet = stomach.get_gullet_mut();
      contents.push_str(&untex(Expand!(tokens,gullet,state),false)?);
      contents.push('\n');
      AssignValue!(&handle => contents, Some(Scope::Global));
    } else {
      let gullet = stomach.get_gullet_mut();
      println_stderr!("{}", untex(Expand!(tokens, gullet),false)?);
    }
  });

  // # Since we don't paginate, we're effectively always "shipping out",
  // # so all operations are \immediate
  DefPrimitive!("\\immediate", None);

  // #======================================================================
  // # Remaining semi- Vertical Mode primitives in Ch.24, pp.280--281

  DefPrimitive!("\\special {}", None);
  DefPrimitive!("\\penalty Number", None);
  DefPrimitive!("\\kern Dimension", None);
  DefMacro!("\\mkern MuGlue", "\\ifmmode\\@math@mskip #1\\relax\\else\\@text@mskip #1\\relax\\fi");
  DefPrimitive!("\\unpenalty", None);
  DefPrimitive!("\\unkern", None);
  // ## Worrisome, but...
  // DefPrimitiveI("\unskip", None, sub {
  //     my ($stomach) = @_;
  //     my $box;
  //     while (($box = $LaTeXML::LIST[-1]) && IsEmpty($box)) {
  //       pop(@LaTeXML::LIST); }
  //     return; });

  // DefPrimitive!("\\mark{}", None);
  // # \insert<8bit><filler>{<vertical mode material>}
  DefPrimitive!("\\insert Number", None); // Just let the insertion get processed(?)
                                          // \vadjust<filler>{<vertical mode material>}
                                          // Note: \vadjust ignores in vertical mode...
                                          // is it sufficient to just clear the macro to avoid recursion?
                                          // (we don't track horizontal/vertical mode)
  DefMacro!("\\LTX@vadjust@afterpar", "\\def\\LTX@vadjust@afterpar{}");
  DefMacro!(
    "\\LTX@clear@vadjust@afterpar",
    "\\def\\LTX@vadjust@afterpar{\\def\\LTX@vadjust@afterpar{}}"
  );
  DefPrimitive!("\\vadjust {}", sub[stomach,(arg),state] {
    state.push_tokens("vAdjust", arg);
  });

  // #======================================================================
  // # Remaining Vertical Mode primitives in Ch.24, pp.281--283
  // # \vskip<glue>, \vfil, \vfill, \vss, \vfilneg
  // # <leaders> = \leaders | \cleaders | \xleaders
  // # <box or rule> = <box> | <vertical rule> | <horizontal rule>
  // # <vertical rule> = \vrule<rule specification>
  // # <horizontal rule> = \hrule<rule specification>
  // # <rule specification> = <optional spaces> | <rule dimension><rule specification>
  // # <rule dimension> = width <dimen> | height <dimen> | depth <dimen>

  // # Stuff to ignore for now...
  DefPrimitive!("\\vfil", None);
  DefPrimitive!("\\vfill", None);
  DefPrimitive!("\\vss", None);
  DefPrimitive!("\\vfilneg", None);
  DefPrimitive!("\\leaders", None);
  DefPrimitive!("\\cleaders", None);
  DefPrimitive!("\\xleaders", None);

  // # \moveleft<dimen><box>, \moveright<dimen><box>
  // DefConstructor('\moveleft Dimension MoveableBox',
  //   "<ltx:text xoffset='#x' _noautoclose='1'>#2</ltx:text>",
  //   afterDigest => sub {
  //     $_[1]->setProperty(x => $_[1]->getArg(1)->multiply(-1)); });
  // DefConstructor('\moveright Dimension MoveableBox',
  //   "<ltx:text xoffset='#x' _noautoclose='1'>#2</ltx:text>",
  //   afterDigest => sub {
  //     $_[1]->setProperty(x => $_[1]->getArg(1)); });

  // # \unvbox<8bit>, \unvcopy<8bit>
  // DefPrimitive('\unvbox Number', sub {
  //     my $box   = 'box' . $_[1]->valueOf;
  //     my $stuff = LookupValue($box);
  //     AssignValue($box, undef);
  //     (defined $stuff ? $stuff->unlist : ()); });
  // DefPrimitive('\unvcopy Number', sub {
  //     my $box   = 'box' . $_[1]->valueOf;
  //     my $stuff = LookupValue($box);
  //     (defined $stuff ? $stuff->unlist : ()); });

  //======================================================================
  // If this is the right solution...
  // then we also should put the desired spacing on a style attribute?!?!?!
  DefConstructor!("\\vskip Glue", sub[document, args, props, state] {
    unref!(args => length);
    let length = length.pt_value(None);

    if length > 10.0 {    // Or what!?!?!?!
      if document.is_closeable("ltx:para", state).is_some() {
        document.close_element("ltx:para", state)?;
      } else if document.is_openable("ltx:break", state) {
        document.insert_element("ltx:break", Vec::new(), None, state)?;
      }
    }},
    properties => {map!("isSpace" => true.into(), "isVerticalSpace" => true.into())}
  );
});
