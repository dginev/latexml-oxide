use crate::package::*;
use rtx_core::list::List;
use rtx_core::tbox::Tbox;
use rtx_core::TexMode;
// use super::tex_boxes::adjust_box_color;

static PSFILE_REGEX : Lazy<Regex> = Lazy::new(|| Regex::new(r"\bpsfile=(.+?)(?:\s|\})").unwrap());

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
    let open = Tbox::new(arena::pin_static(""), None, None,
        Tokens!(T_BEGIN!()), stored_map!("isEmpty" => true), state);
    let mode = Some(if state.lookup_bool("IN_MATH") { TexMode::Math} else {TexMode::Text});
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
      properties: HashMap::default()
    }
  });

  DefPrimitive!(
    "}",
    sub[stomach, (), state] {
      let f = LookupFont!();
      stomach.egroup(state)?;
      Tbox::new(arena::pin_static(""), f, None, Tokens!(T_END!()), stored_map!("isEmpty"=>true), state)
    }
  );

  // These are for those screwy cases where you need to create a group like box,
  // more than just bgroup, egroup,
  // BUT you DON'T want extra {, } showing up in any untex-ing.
  DefConstructor!("\\@hidden@bgroup", "#body",
    before_digest => sub[stomach,state] { stomach.bgroup(state); },
    capture_body => true,
    reversion=> sub[whatsit, _args,state] {
      if let Some(body) = whatsit.get_body() {
        body.revert(state)
      } else { Ok(Tokens!()) }
    }
  );
  DefConstructor!("\\@hidden@egroup", "",
    after_digest => sub[stomach,_args,state] { stomach.egroup(state)?; },
    reversion => ""
  );

  DefPrimitive!(
  "\\begingroup", sub[stomach, _args, state] {
    stomach.begingroup(state);
  });
  DefPrimitive!(
  "\\endgroup", sub[stomach, _args, state] {
    stomach.endgroup(state)?;
  });

  // Debugging aids; Ignored!
  DefPrimitive!("\\show Token", sub[stomach,(arg),state] {
    let mut gullet = stomach.get_gullet_mut();
    let lhs = if arg.get_catcode() == Catcode::CS {
      s!("{arg}=")
    } else { String::new() };
    let stuff = Invocation!(T_CS!("\\meaning"), vec![arg], gullet)?;
    let rhs = writable_tokens(&Expand!(stuff, gullet), state);
    // TODO: add+use `Note!` instead of `eprintln`
    eprintln!("> {lhs}{rhs}");
    eprintln!("{}",gullet.get_locator().unwrap_or_default());
    ()
  });
  DefPrimitive!("\\showbox Number", sub[stomach,(arg),state] {
    let n     = arg.value_of();
    let stuff = state.lookup_value(&s!("box{n}"));
    Debug!("Box {n} = {stuff:?}");
    ()
  });
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
            Cow::Owned(Stored::Bool(ref space_bool))  => *space_bool,
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
  // \aftergroup saves ALL tokens (from repeated calls) to be executed IN ORDER after the next
  // egroup or }
  DefPrimitive!("\\aftergroup Token", sub[stomach, (t), state] {
    state.push_value("afterGroup", t);
  });

  // \uppercase<general text>, \lowercase<general text>

  // Note that these are NOT expandable, even though the "return" tokens!
  DefPrimitive!("\\uppercase GeneralText", sub[stomach,(tokens), state] {
    stomach.get_gullet_mut().unread_vec(
      tokens.unlist().into_iter()
        .map(|t| uppercase_token(t, state))
        .collect());
  });
  DefPrimitive!("\\lowercase GeneralText", sub[stomach,(tokens), state] {
    stomach.get_gullet_mut().unread_vec(
      tokens.unlist().into_iter()
        .map(|t| lowercase_token(t, state))
        .collect::<Vec<Token>>());
  });

  DefPrimitive!("\\message{}", sub [stomach, (message), state] {
    if state.lookup_int("VERBOSITY") > -1 {
      eprintln!("{}", writable_tokens(
        &do_expand(message, stomach.get_gullet_mut(), state)?, state));
    }
  });

  DefRegister!("\\errhelp", Tokens!());
  DefPrimitive!("\\errmessage{}", sub[stomach,(args),state] {
    let mut gullet = stomach.get_gullet_mut();
    let message = Expand!(args, gullet, state);
    let help = Expand!(Tokens!(T_CS!("\\the"), T_CS!("\\errhelp")), gullet, state);
    eprintln!("{}: {}", message, help);
  });

  // TeX I/O primitives
  DefPrimitive!("\\openin Number SkipMatch:= SkipSpaces TeXFileName",
  sub[stomach, (port, filename), state] {
    let port = port.to_string();
    let filename = filename.to_string();
    // possibly should close $port if it's already been opened?
    // Rely on FindFile to enforce any access restrictions
    if let Some(path) = find_file(&filename, Some(
      FindFileOptions {forbid_ltxml: true, ..FindFileOptions::default()}), state) {
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
    let file_key = s!("input_file:{}", port);
    let mouth_opt = if let Some(Stored::Mouth(ref mouth)) = LookupValue!(&file_key) {
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

  DefPrimitive!("\\read Number SkipKeyword:to SkipSpaces Token",
    sub[stomach, (port, token), state] {
    if let Some(Stored::Mouth(mouth_stored)) = state.lookup_value(&format!("input_file:{port}")) {
      let mouth_obj = Rc::clone(mouth_stored);
      stomach.bgroup(state);
      AssignValue!("PRESERVE_NEWLINES", 2); // Special EOL/EOF treatment for \read
      AssignValue!("INCLUDE_COMMENTS", false);
      let mut tokens = Vec::new();
      let mut level = 0;
      let mut mouth = mouth_obj.borrow_mut();
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
      mouth.borrow().at_eof()
    } else {
      true
    }
  });

  // For output files, we'll write the data to a cached internal copy
  // rather than to the actual file system.
  DefPrimitive!("\\openout Number SkipMatch:= SkipSpaces TeXFileName",
    sub[stomach, (port, filename), state] {
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
      contents.push_str(&Expand!(tokens,gullet,state).untex());
      contents.push('\n');
      AssignValue!(&handle => contents, Some(Scope::Global));
    } else {
      let gullet = stomach.get_gullet_mut();
      println_stderr!("{}", Expand!(tokens, gullet).untex());
    }
  });

  // Since we don't paginate, we're effectively always "shipping out",
  // so all operations are \immediate
  DefPrimitive!("\\immediate", None);

  //======================================================================
  // Remaining semi- Vertical Mode primitives in Ch.24, pp.280--281

  DefPrimitive!("\\special {}", sub[stomach,(arg),state] {
    let special_str = arg.to_string();
    // recognize one special graphics inclusion case
    if let Some(cap) = PSFILE_REGEX.captures(&special_str) {
      let graphic = cap.get(1).unwrap().as_str();
      RequirePackage!("graphicx", searchpaths_only => true);
      let mut kv = Vec::new();
      for prop in ["voffset","hoffset","hscale","vscale","hsize","vsize","angle"] {
        let prop_regex = Regex::new(&s!("\\b{prop}=(.+?)(?:\\s|\\}})")).unwrap();
        if let Some(cap) = prop_regex.captures(&special_str) {
          let prop_val = cap.get(1).unwrap().as_str();
          if !kv.is_empty() {
            kv.push(T_OTHER!(","));
          }
          kv.push(T_OTHER!(prop));
          kv.push(T_OTHER!("="));
          kv.push(T_OTHER!(prop_val));
        }
      }
      if !kv.is_empty() {
        let mut wrapped = vec![T_OTHER!("[")];
        wrapped.extend(kv);
        wrapped.push(T_OTHER!("]"));
        kv = wrapped;
      }
      let mut gullet = stomach.get_gullet_mut();
      gullet.unread_vec(vec![T_BEGIN!(), T_OTHER!(graphic), T_END!()]);
      gullet.unread_vec(kv);
      gullet.unread_one(T_CS!("\\ltx@special@graphics"));
    } else {
      Info!("ignored", "special", stomach, state, s!("Unrecognized TeX Special: {arg}"));
    }
  });

  // # adapted from graphicx.sty.ltxml
  // DefKeyVal('SpecialPS', 'angle',   '');
  // DefKeyVal('SpecialPS', 'voffset', '');
  // DefKeyVal('SpecialPS', 'hoffset', '');
  // DefKeyVal('SpecialPS', 'hsize',   '');
  // DefKeyVal('SpecialPS', 'vsize',   '');
  // DefKeyVal('SpecialPS', 'hscale',  '');
  // DefKeyVal('SpecialPS', 'vscale',  '');
  // DefConstructor('\ltx@special@graphics OptionalKeyVals:SpecialPS Semiverbatim',
  //   "<ltx:graphics graphic='#path' candidates='#candidates' options='#options'/>",
  //   sizer      => \&image_graphicx_sizer,
  //   properties => sub {
  //     my ($stomach, $kv, $path) = @_;
  //     $path = ToString($path); $path =~ s/^\s+//; $path =~ s/\s+$//;
  //     $path =~ s/("+)(.+)\g1/$2/;
  //     my $searchpaths = LookupValue('GRAPHICSPATHS');
  //     my @candidates  = pathname_findall($path, types => ['*'], paths => $searchpaths);
  //     if (my $base = LookupValue('SOURCEDIRECTORY')) {
  //       @candidates = map { pathname_relative($_, $base) } @candidates; }
  //     my $options = '';
  //     if ($kv) {    # remap psfile options to includegraphics options:
  //       if (my $hscale = $kv->getValue('hscale')) {
  //         $hscale = $hscale && int(ToString($hscale)) / 100;
  //         $options .= ',' if $options;
  //         $options .= "xscale=$hscale"; }
  //       if (my $vscale = $kv->getValue('vscale')) {
  //         $vscale = $vscale && int(ToString($vscale)) / 100;
  //         $options .= ',' if $options;
  //         $options .= "yscale=$vscale"; }
  //       if (my $hsize = $kv->getValue('hsize')) {
  //         $hsize = ToString($hsize);
  //         $options .= ',' if $options;
  //         $options .= "width=$hsize"; }
  //       if (my $vsize = $kv->getValue('vsize')) {
  //         $vsize = ToString($vsize);
  //         $options .= ',' if $options;
  //         $options .= "height=$vsize"; }
  //       if (my $angle = $kv->getValue('angle')) {
  //         $angle = ToString($angle);
  //         $options .= ',' if $options;
  //         $options .= "angle=$angle"; }
  //       my $voffset = $kv->getValue('voffset') || 0;
  //       $voffset = $voffset && int(ToString($voffset));
  //       my $hoffset = $kv->getValue('hoffset') || 0;
  //       $hoffset = $hoffset && int(ToString($hoffset));
  //       if ($voffset || $hoffset) {
  //         my $left   = -$hoffset;
  //         my $bottom = -$voffset;
  //         $options .= "," if $options;
  //         $options .= "trim=$left $bottom 0 0,clip=true"; } }
  //     (options => $options, path => $path, candidates => join(',', @candidates)); },
  //   mode => 'text');
  // # Since these ultimately generate external resources, it can be useful to have a handle on them.
  // Tag('ltx:graphics', afterOpen => sub { GenerateID(@_, 'g'); });

  DefPrimitive!("\\penalty Number", None);
  // \kern is heavily used by xy.
  // Completely HACK version for the moment
  // Note that \kern should add vertical spacing in vertical modes!
  DefConstructor!("\\kern Dimension", sub[document,args,state] {
    let length = if let DigestedData::RegisterValue(RegisterValue::Dimension(d)) =
      args[0].as_ref().unwrap().data() {
        *d
      } else { Dimension::default() };
      let is_svg_g = document.with_node_qname(document.get_node(), state,
        |qname| qname == "svg:g");
    let parent = document.get_node_mut();
    if is_svg_g {
      let x = length.px_value(None);
      if x > 0.0 {
        // HACK HACK HACK
        let mut transform = parent.get_attribute("transform").unwrap_or_default();
        if !transform.is_empty() {
          transform.push(' ');
        }
        transform.push_str(&s!("translate({x},0)"));
        parent.set_attribute("transform", &transform)?;
      }
    } else if in_svg(document,state) {
      Warn!("unexpected", "kern", document, state, s!("Lost kern in SVG {length}"));
    }
  });
  DefMacro!(
    "\\mkern MuGlue",
    "\\ifmmode\\@math@mskip #1\\relax\\else\\@text@mskip #1\\relax\\fi"
  );
  DefPrimitive!("\\unpenalty", None);
  DefPrimitive!("\\unkern", None);
  // Worrisome, but...
  DefPrimitive!("\\unskip", sub[stomach,(),state] {
    // pop until a non-empty box is found
    while let Some(last_box) = stomach.box_list.pop() {
      if !last_box.is_empty() {
        stomach.box_list.push(last_box);
        break;
      }
    }
  });

  DefPrimitive!("\\mark{}", None);
  // \insert<8bit><filler>{<vertical mode material>}
  DefPrimitive!("\\insert Number", None);
  // \vadjust<filler>{<vertical mode material>}
  // Note: \vadjust ignores in vertical mode...
  DefPrimitive!("\\vadjust {}", sub[stomach,(arg),state] {
    state.push_tokens("vAdjust", arg);
  });

  //======================================================================
  // Remaining Vertical Mode primitives in Ch.24, pp.281--283
  // \vskip<glue>, \vfil, \vfill, \vss, \vfilneg
  // <leaders> = \leaders | \cleaders | \xleaders
  // <box or rule> = <box> | <vertical rule> | <horizontal rule>
  // <vertical rule> = \vrule<rule specification>
  // <horizontal rule> = \hrule<rule specification>
  // <rule specification> = <optional spaces> | <rule dimension><rule specification>
  // <rule dimension> = width <dimen> | height <dimen> | depth <dimen>

  // Stuff to ignore for now...
  DefPrimitive!("\\vfil", None);
  DefPrimitive!("\\vfill", None);
  DefPrimitive!("\\vss", None);
  DefPrimitive!("\\vfilneg", None);
  DefPrimitive!("\\leaders", None);
  DefPrimitive!("\\cleaders", None);
  DefPrimitive!("\\xleaders", None);

  // \moveleft<dimen><box>, \moveright<dimen><box>
  DefConstructor!("\\moveleft Dimension MoveableBox",
    "<ltx:text xoffset='#x' _noautoclose='true'>#2</ltx:text>",
    after_digest => sub[_stomach,whatsit,_state] {
      if let DigestedData::RegisterValue(d) = whatsit.get_arg(1).unwrap().data() {
        whatsit.set_property("x", d.clone().multiply(Number::new(-1)));
      }});
  DefConstructor!("\\moveright Dimension MoveableBox",
    "<ltx:text xoffset='#x' _noautoclose='true'>#2</ltx:text>",
    after_digest => sub[_stomach,whatsit,_state] {
      if let Some(dimension) = whatsit.get_arg(1) {
        whatsit.set_property("x", dimension.clone());
      }});

  // DG: TODO: We need tests+examples here, a bit lost in the typing interface...
  //
  // # \unvbox<8bit>, \unvcopy<8bit>
  // DefPrimitive!("\\unvbox Number", sub[stomach,(number),state] {
  //   let box_key   = s!("box{}",number.value_of());
  //   let stuff = state.lookup_tokens(&box_key);
  //   adjust_box_color(stuff, state);
  //   AssignValue!(&box_key, None);
  //   stuff.map(|tks| Digested::from(tks)).unwrap_or_else(|| Digested::from(List::default()))
  // });
  // DefPrimitive!("\\unvcopy Number", sub[stomach,(number),state] {
  //   let box_key   = s!("box{}",number.value_of());
  //   let stuff = state.lookup_tokens(&box_key);
  //   adjust_box_color(stuff, state);
  //   stuff.map(|tks| Digested::from(tks)).unwrap_or_else(|| Digested::from(List::default()))
  // });


  //======================================================================
  // If this is the right solution...
  // then we also should put the desired spacing on a style attribute?!?!?!
  DefConstructor!("\\vskip Glue", sub[document, args, _props, state] {
    unref!(args => length);
    let length = length.pt_value(None);

    if length > 10.0 {    // Or what!?!?!?!
      if document.is_closeable("ltx:para", state).is_some() {
        document.close_element("ltx:para", state)?;
      } else if document.is_openable("ltx:break", state) {
        document.insert_element("ltx:break", Vec::new(), None, state)?;
      }
    }},
     // TODO: "height" property
    properties => {stored_map!("isSpace" => true, "isVerticalSpace" => true, "isBreak" => true)}
  );
});
