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

LoadDefinitions!({
  //======================================================================
  // Remaining Mode independent primitives in Ch.24, pp.279-280
  // \relax was done as expandable (isn't that right?)
  // }
  // Note, we don't bother making sure begingroup is ended by endgroup.

  // These define the handler for { } (or anything of catcode BEGIN, END)

  // These are actually TeX primitives, but we treat them as a Whatsit so they
  // remain in the constructed tree.
  DefPrimitive!("{", sub[()] {
    stomach_mut!().bgroup();
    let open = Tbox::new(arena::pin_static(""), None, None,
        Tokens!(T_BEGIN!()), stored_map!("isEmpty" => true));
    let mode = Some(if lookup_bool("IN_MATH") { TexMode::Math} else {TexMode::Text});
    let body = stomach::digest_next_body(None)?;
    let mut boxes = vec![Digested::from(open)];
    boxes.extend(body);
    let mut font = None;
    for abox in boxes.iter().rev() {
      if let Some(boxfont) = abox.get_font()? {
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

  DefPrimitive!("}", {
    let f = LookupFont!();
    stomach_mut!().egroup()?;
    Tbox::new(arena::pin_static(""), f, None, Tokens!(T_END!()), stored_map!("isEmpty"=>true))
  });

  // These are for those screwy cases where you need to create a group like box,
  // more than just bgroup, egroup,
  // BUT you DON'T want extra {, } showing up in any untex-ing.
  DefConstructor!("\\@hidden@bgroup", "#body",
    before_digest => { stomach_mut!().bgroup(); },
    capture_body => true,
    reversion=> sub[whatsit,_args] {
      if let Some(body) = whatsit.get_body()? {
        body.revert()
      } else { Ok(Tokens!()) }
    }
  );
  DefConstructor!("\\@hidden@egroup", "",
    after_digest => { stomach_mut!().egroup()?; },
    reversion => ""
  );

  DefPrimitive!(
  "\\begingroup", {
    stomach_mut!().begingroup();
  });
  DefPrimitive!(
  "\\endgroup", {
    stomach_mut!().endgroup()?;
  });

  // Debugging aids; Ignored!
  DefPrimitive!("\\show Token", sub[(arg)] {
    let mut gullet = gullet_mut!();
    let lhs = if arg.get_catcode() == Catcode::CS {
      s!("{arg}=")
    } else { String::new() };
    let stuff = Invocation!(T_CS!("\\meaning"), vec![arg]);
    let rhs = writable_tokens(&Expand!(stuff));
    // TODO: add+use `Note!` instead of `eprintln`
    eprintln!("> {lhs}{rhs}");
    eprintln!("{}",gullet.get_locator().unwrap_or_default());
  });
  DefPrimitive!("\\showbox Number", sub[(arg)] {
    let n     = arg.value_of();
    Debug!("Box {n} = {:?}", state!().lookup_value(&s!("box{n}")));
  });
  DefPrimitive!("\\showlists", None);
  DefPrimitive!("\\showthe Token", None);

  // DefPrimitive('\shipout ??
  DefPrimitive!("\\ignorespaces SkipSpaces", None);

  DefPrimitive!("\\lx@ignorehardspaces", {
    let mut boxes = Vec::new();
    while let Some(token) = gullet::read_x_token(None, false)? {
      boxes = stomach::invoke_token(&token)?;
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
  DefPrimitive!("\\afterassignment Token", sub[(t)] {
    state::assign_value("afterAssignment", t, Some(Scope::Global));
  });
  // \aftergroup saves ALL tokens (from repeated calls) to be executed IN ORDER after the next
  // egroup or }
  DefPrimitive!("\\aftergroup Token", sub[(t)] {
    push_value("afterGroup", t)
  });

  // \uppercase<general text>, \lowercase<general text>

  // Note that these are NOT expandable, even though the "return" tokens!
  DefPrimitive!("\\uppercase GeneralText", sub[(tokens)] {
    gullet::unread_vec(
      tokens.unlist().into_iter()
        .map(uppercase_token)
        .collect());
  });
  DefPrimitive!("\\lowercase GeneralText", sub[(tokens)] {
    gullet::unread_vec(
      tokens.unlist().into_iter()
        .map(lowercase_token)
        .collect::<Vec<Token>>());
  });

  DefPrimitive!("\\message{}", sub [(message)] {
    if lookup_int("VERBOSITY") > -1 {
      eprintln!("{}", writable_tokens(&do_expand(message)?));
    }
  });

  DefRegister!("\\errhelp", Tokens!());
  DefPrimitive!("\\errmessage{}", sub[(args)] {
    let mut gullet = gullet_mut!();
    let message = Expand!(args);
    let help = Expand!(Tokens!(T_CS!("\\the"), T_CS!("\\errhelp")));
    eprintln!("{}: {}", message, help);
  });

  // TeX I/O primitives
  DefPrimitive!("\\openin Number SkipMatch:= SkipSpaces TeXFileName",
  sub[(port, filename)] {
    let port = port.to_string();
    let filename = filename.to_string();
    // possibly should close $port if it's already been opened?
    // Rely on FindFile to enforce any access restrictions
    if let Some(path) = find_file(&filename, Some(
      FindFileOptions {forbid_ltxml: true, ..FindFileOptions::default()})) {
      let content_str = LookupString!(&s!("{}_contents",path));
      let content = if content_str.is_empty() {
        None
      } else {
        Some(content_str)
      };
      let mouth = Mouth::create(&path, MouthOptions {
        content,
        .. MouthOptions::default()
      })?;
      AssignValue!(&s!("input_file:{}", port), mouth, Some(Scope::Global));
    }
  });

  DefPrimitive!("\\closein Number", sub[(port)] {
    // Clone the Rc<> for mouth out of state:: since we'll be mutating.
    let file_key = s!("input_file:{}", port);
    let mouth_opt = if let Some(Stored::Mouth(ref mouth)) = state!().lookup_value(&file_key) {
      Some(Rc::clone(mouth))
    } else {
      None
    };
    //   close the mouth (if any) and clear the variable
    if let Some(mouth) = mouth_opt {
      mouth.borrow_mut().finish();
      AssignValue!(&s!("input_file:{}", port), false, Some(Scope::Global));
    }
  });

  DefPrimitive!("\\read Number SkipKeyword:to SkipSpaces Token",
    sub[(port, token)] {
    if let Some(Stored::Mouth(mouth_stored)) = state!().lookup_value(&format!("input_file:{port}")) {
      let mouth_obj = Rc::clone(mouth_stored);
      stomach_mut!().bgroup();
      AssignValue!("PRESERVE_NEWLINES", 2); // Special EOL/EOF treatment for \read
      AssignValue!("INCLUDE_COMMENTS", false);
      let mut tokens = Vec::new();
      let mut level = 0;
      let mut mouth = mouth_obj.borrow_mut();
      while let Some(t) = mouth.read_token() {
        let cc = t.get_catcode();
        if cc != Catcode::MARKER {
          tokens.push(t);
        }
        match cc {
          Catcode::BEGIN => {level += 1},
          Catcode::END => {level -= 1},
          _ => {}
        };
        if level == 0 && mouth.is_eol() {
          break;
        }
      }
      stomach_mut!().egroup()?;
      DefMacro!(token, None, Tokens::new(tokens), nopack_parameters => true);
    }
  });

  DefConditional!("\\ifeof Number", sub[(port)] {
    if let Some(Stored::Mouth(mouth)) = state!().lookup_value(&s!("input_file:{}", port)) {
      mouth.borrow().at_eof()
    } else {
      true
    }
  });

  // For output files, we'll write the data to a cached internal copy
  // rather than to the actual file system.
  DefPrimitive!("\\openout Number SkipMatch:= SkipSpaces TeXFileName",
    sub[(port, filename)] {
    let port = port.to_string();
    let filename = filename.to_string();
    let contents_key = &s!("{}_contents",filename);
    AssignValue!(&s!("output_file:{}",port)  => filename,  Some(Scope::Global));
    AssignValue!(contents_key => "",  Some(Scope::Global));
  });

  DefPrimitive!("\\closeout Number", sub[(port)] {
    AssignValue!(&s!("output_file:{}",port), false, Some(Scope::Global));
  });

  DefPrimitive!("\\write Number {}", sub[(port, tokens)] {
    if let Some(filename) = state!().lookup_value(&s!("output_file:{}", port)) {
      let handle   = s!("{}_contents",filename);
      let mut contents : String = LookupString!(&handle);
      let mut gullet = gullet_mut!();
      contents.push_str(&Expand!(tokens).untex());
      contents.push('\n');
      AssignValue!(&handle => contents, Some(Scope::Global));
    } else {
      let mut gullet = gullet_mut!();
      println_stderr!("{}", Expand!(tokens).untex());
    }
  });

  // Since we don't paginate, we're effectively always "shipping out",
  // so all operations are \immediate
  DefPrimitive!("\\immediate", None);

  //======================================================================
  // Remaining semi- Vertical Mode primitives in Ch.24, pp.280--281

  DefPrimitive!("\\special {}", sub[(arg)] {
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
      let mut gullet = gullet_mut!();
      gullet.unread_vec(vec![T_BEGIN!(), T_OTHER!(graphic), T_END!()]);
      gullet.unread_vec(kv);
      gullet.unread_one(T_CS!("\\ltx@special@graphics"));
    } else {
      Info!("ignored", "special", s!("Unrecognized TeX Special: {arg}"));
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
  DefConstructor!("\\kern Dimension", sub[document,args] {
    let length = if let DigestedData::RegisterValue(RegisterValue::Dimension(d)) =
      args[0].as_ref().unwrap().data() {
        *d
      } else { Dimension::default() };
      let is_svg_g = document::with_node_qname(document.get_node(),
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
    } else if in_svg(document) {
      Warn!("unexpected", "kern", s!("Lost kern in SVG {length}"));
    }
  });
  DefMacro!(
    "\\mkern MuGlue",
    "\\ifmmode\\@math@mskip #1\\relax\\else\\@text@mskip #1\\relax\\fi"
  );
  DefPrimitive!("\\unpenalty", None);
  DefPrimitive!("\\unkern", None);
  // Worrisome, but...
  DefPrimitive!("\\unskip", {
    // pop until a non-empty box is found
    while let Some(last_box) = stomach_mut!().box_list.pop() {
      if !last_box.is_empty()? {
        stomach_mut!().box_list.push(last_box);
        break;
      }
    }
  });

  DefPrimitive!("\\mark{}", None);
  // \insert<8bit><filler>{<vertical mode material>}
  DefPrimitive!("\\insert Number", None);
  // \vadjust<filler>{<vertical mode material>}
  // Note: \vadjust ignores in vertical mode...
  DefPrimitive!("\\vadjust {}", sub[(arg)] { push_tokens("vAdjust", arg); });

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
    after_digest => sub[whatsit] {
      if let DigestedData::RegisterValue(d) = whatsit.get_arg(1).unwrap().data() {
        whatsit.set_property("x", d.clone().multiply(Number::new(-1)));
      }});
  DefConstructor!("\\moveright Dimension MoveableBox",
    "<ltx:text xoffset='#x' _noautoclose='true'>#2</ltx:text>",
    after_digest => sub[whatsit] {
      if let Some(dimension) = whatsit.get_arg(1) {
        whatsit.set_property("x", dimension.clone());
      }});

  // DG: TODO: We need tests+examples here, a bit lost in the typing interface...
  //
  // # \unvbox<8bit>, \unvcopy<8bit>
  // DefPrimitive!("\\unvbox Number", sub[(number)] {
  //   let box_key   = s!("box{}",number.value_of());
  //   let stuff = state!().lookup_tokens(&box_key);
  //   adjust_box_color(stuff);
  //   AssignValue!(&box_key, None);
  //   stuff.map(|tks| Digested::from(tks)).unwrap_or_else(|| Digested::from(List::default()))
  // });
  // DefPrimitive!("\\unvcopy Number", sub[(number)] {
  //   let box_key   = s!("box{}",number.value_of());
  //   let stuff = state!().lookup_tokens(&box_key);
  //   adjust_box_color(stuff);
  //   stuff.map(|tks| Digested::from(tks)).unwrap_or_else(|| Digested::from(List::default()))
  // });


  //======================================================================
  // If this is the right solution...
  // then we also should put the desired spacing on a style attribute?!?!?!
  DefConstructor!("\\vskip Glue", sub[document, args, _props] {
    unref!(args => length);
    let length = length.pt_value(None);

    if length > 10.0 {    // Or what!?!?!?!
      if document.is_closeable("ltx:para").is_some() {
        document.close_element("ltx:para")?;
      } else if document.is_openable("ltx:break") {
        document.insert_element("ltx:break", Vec::new(), None)?;
      }
    }},
     // TODO: "height" property
    properties => {stored_map!("isSpace" => true, "isVerticalSpace" => true, "isBreak" => true)}
  );
});
