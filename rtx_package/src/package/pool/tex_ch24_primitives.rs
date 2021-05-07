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
  DefPrimitive!("{", sub[stomach, args, state] {
    stomach.bgroup(state);
    let open = Tbox::new(String::new(), None, None, Tokens!(T_BEGIN!()), HashMap::new(), state);
    let mode = if LookupBool!("IN_MATH") {
      Some(TexMode::Math)
    } else {
      Some(TexMode::Text)
    };
    let body = stomach.digest_next_body(None, state)?;
    let mut boxes = vec![Digested::TBox(Rc::new(open))];
    boxes.extend(body);
    // TODO: Locator logic here needs to improve..
    List {
      boxes,
      mode,
      font: None,
      locator: Locator::default(),
    }
  });

  DefPrimitive!(
    "}",
    sub[stomach, args, state] {
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
    capture_body => true
    // TODO: // reversion => sub { Revert($_[0]->getProperty("body")); }
  );
  DefConstructor!("\\@hidden@egroup", "",
    after_digest => sub[stomach,args,state] { stomach.egroup(state)?; },
    reversion => None
  );

  DefPrimitive!(
  "\\begingroup", sub[stomach, _args, state] {
    stomach.begingroup(state);
  });
  DefPrimitive!(
  "\\endgroup", sub[stomach, _args, state] {
    stomach.endgroup(state)?;
  });

  // // Debugging aids; Ignored!
  DefPrimitive!("\\show Token", None);
  DefPrimitive!("\\showbox Number", None);
  DefPrimitive!("\\showlists", None);
  DefPrimitive!("\\showthe Token", None);

  // DefPrimitive('\shipout ??
  DefPrimitive!("\\ignorespaces SkipSpaces", None);

  DefPrimitive!("\\lx@ignorehardspaces", sub[stomach, whatsit, state] {
    let mut boxes = Vec::new();
    while let Some(token) = stomach.get_gullet_mut().read_x_token(false, false, state)? {
      boxes = stomach.invoke_token(&token, state)?;
      if boxes.is_empty() {
        break;
      }
      while !boxes.is_empty() {
        let is_space = if let Some(space_val) = boxes[0].get_property("isSpace", state) {
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
  DefPrimitive!("\\afterassignment Token", sub[stomach, args, state] {
    unpack_to_token!(args => t);
    AssignValue!("afterAssignment" => t, Some(Scope::Global)); });
  // \aftergroup saves ALL tokens (from repeated calls) to be executed IN ORDER after the next egroup or }
  DefPrimitive!("\\aftergroup Token", sub[stomach, args, state] {
    unpack_to_token!(args => t);
    PushValue!("afterGroup" => t); });

  // // \uppercase<general text>, \lowercase<general text>
  // sub ucToken {
  //   my ($token) = @_;
  //   my $code = $STATE->lookupUCcode($token->getString);
  //   return ((defined $code) && ($code != 0) ? Token(chr($code), $token->getCatcode) : $token); }

  // sub lcToken {
  //   my ($token) = @_;
  //   my $code = $STATE->lookupLCcode($token->getString);
  //   return ((defined $code) && ($code != 0) ? Token(chr($code), $token->getCatcode) : $token); }

  // DefMacro('\uppercase GeneralText', sub {
  //     my ($gullet, $tokens) = @_;
  //     return map { ucToken($_) } $tokens->unlist; });

  // DefMacro('\lowercase GeneralText', sub {
  //     my ($gullet, $tokens) = @_;
  //     return map { lcToken($_) } $tokens->unlist; });

  // DefPrimitive('\message{}', sub {
  //     my ($stomach, $stuff) = @_;
  //     print STDERR ToString(Expand($stuff)) . "\n" if LookupValue('VERBOSITY') > -1;
  //     return; });

  // DefRegister('\errhelp' => Tokens());
  // DefPrimitive('\errmessage{}', sub {
  //     my ($stomach, $stuff) = @_;
  // print STDERR ToString(Expand($stuff)) . ": " . ToString(Expand(Tokens(T_CS('\the'),
  // T_CS('\errhelp')))) . "\n";     return; });

  // TeX I/O primitives
  DefPrimitive!("\\openin Number SkipMatch:= SkipSpaces TeXFileName", sub[stomach, args, state] {
    unpack!(args => port, filename);
    // possibly should close $port if it's already been opened?
    let port = port.to_string();
    let filename = filename.to_string();
    // Rely on FindFile to enforce any access restrictions
    if let Some(path) = find_file(&filename, None, state) {
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

  DefPrimitive!("\\closein Number", sub[stomach, args, state] {
    unpack!(args => port);
    // Clone the Rc<> for mouth out of state, since we'll be mutating.
    let mouth_opt = if let Some(Stored::Mouth(ref mouth)) = LookupValue!(&s!("input_file:{}", port)) {
      Some(Rc::clone(mouth))
    } else {
      None
    };
    //   close the mouth (if any) and clear the variable
    if let Some(mouth) = mouth_opt {
      mouth.borrow_mut().finish(state);
      AssignValue!(&s!("input_file:{}", port), false, Some(Scope::Global));
    }
  });

  DefPrimitive!("\\read Number SkipKeyword:to SkipSpaces Token", sub[stomach, args, state] {
    unpack!(args => port, token);
    let token: Token = token.into(); // downcast from Tokens
    let port = port.to_number();
    let mouth_opt = if let Some(Stored::Mouth(mouth)) = LookupValue!(&s!("input_file:{}",port)) {
      Some(Rc::clone(mouth))
    } else {
      None
    }; // need to move out the Rc<RefCell<Mouth>> to reuse state later.
    if let Some(mouth) = mouth_opt {
      stomach.bgroup(state);
      AssignValue!("PRESERVE_NEWLINES", 2);
      let mut tokens = Vec::new();
      let mut level = 0;
      while let Some(t) = mouth.borrow_mut().read_token(state) {
        let cc = t.get_catcode();
        level += match cc {
          Catcode::BEGIN => 1,
          Catcode::END => -1,
          _ => 0
        };
        let finished = level == 0 && (
          (cc == Catcode::SPACE && t.get_string() == "\n")
          || cc == Catcode::COMMENT
          || t == T_CS!("\\par"));
        tokens.push(t);
        if finished {
          break;
        }
      }
      stomach.egroup(state)?;
      if tokens.is_empty() {
        tokens = vec![T_CS!("\\par")]; // trailing blank line
      }
      DefMacro!(token, None, Tokens::new(tokens));
    }
  });

  DefConditional!("\\ifeof Number", sub[gullet, args, state] {
    unpack_to_token!(args => port);
    let port = port.to_number();
    if let Some(Stored::Mouth(mouth)) = LookupValue!(&s!("input_file:{}", port)) {
      mouth.borrow().at_eof()
    } else {
      true
    }
  });

  // For output files, we'll write the data to a cached internal copy
  // rather than to the actual file system.
  DefPrimitive!("\\openout Number SkipMatch:= SkipSpaces TeXFileName", sub[stomach, args, state] {
    unpack_to_string!(args => port, filename);
    let contents_key = &s!("{}_contents",filename);
    AssignValue!(&s!("output_file:{}",port)  => filename,  Some(Scope::Global));
    AssignValue!(contents_key => "",  Some(Scope::Global));
  });

  DefPrimitive!("\\closeout Number", sub[stomach, args, state] {
    unpack!(args => port);
    AssignValue!(&s!("output_file:{}",port), false, Some(Scope::Global));
  });

  DefPrimitive!("\\write Number {}", sub[stomach, args, state] {
    unpack!(args => port, tokens);
    let port = port.to_number();
    if let Some(filename) = LookupValue!(&s!("output_file:{}", port)) {
      let handle   = s!("{}_contents",filename);
      let contents = LookupString!(&handle);
      AssignValue!(&handle => s!("{}{}\n", contents, tokens), Some(Scope::Global));
    } else {
      let gullet = stomach.get_gullet_mut();
      println_stderr!("{}\n", Expand!(tokens, gullet));
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
  // DefPrimitive('\vadjust {}', sub {
  //     AddToMacro('\LTX@vadjust@afterpar', $_[1]->unlist);
  //     return; });

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
  // foreach my $op ('\vfil', '\vfill', '\vss', '\vfilneg',
  //   '\leaders', '\cleaders', '\xleaders') {
  //   DefPrimitive($op, undef); }

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
    unpack!(args => length);
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
