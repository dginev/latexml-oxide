//! TeX Box
//! 
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Box Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  //======================================================================
  // These define the handler for { } (or anything of catcode BEGIN, END)

  // These are actually TeX primitives, but we treat them as a Whatsit so they
  // remain in the constructed tree.
  DefPrimitive!("{", {
    bgroup();
    let open = Tbox::new(*EMPTY_SYM, None, None,
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
      properties: SymHashMap::default()
    }
  });

  DefPrimitive!("}", {
    let f = LookupFont!();
    egroup()?;
    Tbox::new(*EMPTY_SYM, f, None, Tokens!(T_END!()), stored_map!("isEmpty"=>true))
  });

  // These are for those screwy cases where you need to create a group like box,
  // more than just bgroup, egroup,
  // BUT you DON'T want extra {, } showing up in any untex-ing.
  DefConstructor!("\\@hidden@bgroup", "#body",
    before_digest => { bgroup(); },
    capture_body => true,
    reversion=> sub[whatsit,_args] {
      if let Some(body) = whatsit.get_body()? {
        body.revert()
      } else { Ok(Tokens!()) }
    }
  );
  DefConstructor!("\\@hidden@egroup", "",
    after_digest => { egroup()?; },
    reversion => ""
  );

  DefMacro!("\\lx@nounicode {}", r"\ifmmode\lx@math@nounicode#1\else\lx@text@nounicode#1\fi");

  DefConstructor!("\\lx@framed[]{}",
    "<ltx:text framed='#frame' _noautoclose='1'>#2</ltx:text>" // TODO
  //   properties => { frame => sub { ToString($_[1] || 'rectangle'); }}
  );
  DefConstructor!("\\lx@hflipped{}",
    "<ltx:text class='ltx_hflipped' _noautoclose='1'>#1</ltx:text>");

  // sub reportNoUnicode {
  //   my ($cs) = @_;
  //   $cs = ToString($cs);
  //   if (!LookupMapping('missing_unicode' => $cs)) {
  //     Warn('expected', 'unicode', $cs,
  //       "There's no Unicode equivalent for the symbol '$cs'");
  //     AssignMapping('missing_unicode' => $cs => 1); }
  //   return; }
  // # Slightly contrived so that this can be used within a DefMath
  // # and still declare & get the semantic properties.
  // DefPrimitive('\lx@math@nounicode DefToken', sub {
  //     my ($stomach, $cs) = @_;
  //     reportNoUnicode($cs);
  //     Box(ToString($cs), undef, undef, $cs, class => 'ltx_nounicode'); });

  // DefConstructor('\lx@text@nounicode DefToken',
  //   "<ltx:text _no_autoclose='true' class='ltx_nounicode'>#1</ltx:text>",
  //   afterDigest => sub {
  //     reportNoUnicode(ToString($_[1]->getArg(0))); });

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Box creation commands
  // ----------------------------------------------------------------------
  // \hbox           c  constructs a box holding horizontal material.
  // \vbox           c  constructs a box holding vertical material.
  // \vtop           c  is an alternate way to construct a box holding vertical material.
  //
  // \everyhbox      pt holds tokens inserted at the start of every hbox.
  // \everyvbox      pt holds tokens inserted at the start of every vbox.
  // ======================================================================
  

  DefParameterType!(BoxSpecification, sub[_inner, _extra] {
    if let Some(key) = gullet::read_keyword(&["to", "spread"])? {
      Ok(Tokens!(T_OTHER!(key)))
    } else {
      Ok(Tokens!())
    }
  },
  // The predigest closure is new for rtx, as it was a single closure in Perl
  // The key problem is that in rtx the parameter type interfaces are well-typed,
  // so it is not possible to remain elegant while at the same time
  // have access to the stomach AND digest.
  // Hence, the `reader` is exclusively responsible for using the gullet to obtain tokens,
  // while early/immediate digestion via the stomach can be achieved
  // by using the separate `predigest` interface
  // Importantly, predigest forces the parameter to be usable
  // only for stomach-capable bindings,
  // namely DefConstructor, DefPrimitive or DefEnvironment
  predigest => sub[key] {
    if !key.is_empty() {
      let mut keyvals = KeyVals::new(
        KeyvalsConfig{skip_missing: keyvals::SkipMissing::All, ..KeyvalsConfig::default()});
      let dim = gullet::read_dimension()?;
      keyvals.set_value(&key.owned_tokens().unwrap().to_string(), dim.into(), false)?;
      keyvals.into()
    } else {
      Ok(None)
    }
  },
  optional => true);

  DefRegister!("\\everyhbox", Tokens!());
  DefRegister!("\\everyvbox", Tokens!());

  DefParameterType!(HBoxContents, sub[_inner, _extra] {
      read_box_contents(state::lookup_tokens("\\everyhbox")) },
    predigest => sub[arg] {
      predigest_box_contents(arg) });
  DefParameterType!(VBoxContents, sub[_inner, _extra] {
      read_box_contents(state::lookup_tokens("\\everyvbox")) },
    predigest => sub[arg] {
      predigest_box_contents(arg) });

  // This re-binds a number of important control sequences to their default text binding.
  // This is useful within common boxing or footnote macros that can appear within
  // alignments or special environments that have redefined many of these.
  AssignValue!("TEXT_MODE_BINDINGS"  => Stored::VecDequeStored(VecDeque::new()));
  AssignValue!("HTEXT_MODE_BINDINGS" => Stored::VecDequeStored(VecDeque::new()));
  AssignValue!("VTEXT_MODE_BINDINGS" => Stored::VecDequeStored(VecDeque::new()));
  push_value("HTEXT_MODE_BINDINGS", Tokens!(T_MATH!(), T_CS!("\\lx@dollar@in@textmode")))?;
  push_value("VTEXT_MODE_BINDINGS", Tokens!(T_MATH!(), T_CS!("\\lx@dollar@in@normalmode")))?;


  // TODO: collapseSVGGroup
  Tag!("svg:g", after_close => sub[_document, _node] {
    Err(unported!())?
    // collapse_svg_group(document, node)
  });


  DefConstructor!("\\hbox BoxSpecification HBoxContents", sub[document, args, props] {
    // "<ltx:text width='#width' _noautoclose='1'>#2</ltx:text>",
    unpack_opt_ref!(args => _spec_opt, contents_opt);
    let contents = contents_opt.as_ref().unwrap();
    let current_opt = document.get_element();

    // What is the CORRECT (& general) way to ask whether we're in "vertical mode"??
    //  my $vmode = $tag eq 'ltx:inline-block'; # ie, explicitly \vbox !?!?!?!
    let is_svg  = if let Some(ref current) = current_opt {
      document::with_node_qname(current, |qname| qname.starts_with("svg:"))
    } else { false };
    let vmode = if let Some(ref current) = current_opt {
      current.has_attribute("_vertical_mode_")
    } else { false };
    let newtag = if is_svg { "svg:g" }
      else if vmode { "ltx:p" } else { "ltx:text" };
    let width : String = if let Some(Stored::Dimension(ref w)) = props.get("width") {
      w.to_attribute()
    } else {
      String::new()
    };
    let node = document.open_element(newtag,
      Some(string_map!("_noautoclose" => "true", "width" => width)), None)?;
    // Note on the clone: Remember that contents is a Digested,
    // i.e. we are cloning an Rc<> wrapper, which is relatively cheap.
    // see the documentation on `Digested` on why we don't have a neater way of dealing with this.
    document.absorb(contents, None)?;
    if !is_svg {
      while !document.get_element().unwrap().has_attribute("_beginscope") &&
        document.maybe_close_element("svg:g")?.is_some() {}
      document.maybe_close_element("svg:svg")?;
      document.maybe_close_node(&node)?;
    } else {
      document.maybe_close_element("svg:g")?;
    }
  },
  mode => "text",
  bounded => true,
  sizer => "#2",
  //   # Workaround for $ in alignment; an explicit \hbox gives us a normal $.
  //   # And also things like \centerline that will end up bumping up to block level!
  before_digest => {
    reenter_text_mode(false)},
  after_digest => sub[whatsit] {
    let width : Option<RegisterValue> = {
      let spec = whatsit.get_arg(1);
      if let Some(ArgWrap::Dimension(w)) = GetKeyVal!(spec, "to") {
        Some((*w).into())
      } else if let Some(ArgWrap::Dimension(s_num_ref)) = GetKeyVal!(spec, "spread") {
        let s_num = *s_num_ref;
        let tbox = whatsit.get_arg_mut(2).unwrap();
        let current_w = tbox.get_width(None)?.unwrap();
        let new_w = current_w.add(s_num);
        Some( new_w )
      } else {
        None
      }
    };
    if let Some(w) = width {
      whatsit.set_width(w);
    }
  });

  // TODO:
  // Tag('svg:foreignObject', autoOpen => 1, autoClose => 1, ...

  DefConstructor!("\\vbox BoxSpecification VBoxContents", sub[document, args, _props] {
      let contents = args[1].as_ref().unwrap();
      let _block = insert_block(document, contents, string_map!("vattach" => "bottom"));
    },
    sizer       => "#2",
    mode        => "text",
    after_digest => sub[_whatsit] {
      // TODO: Height arith
        // let spec = whatsit.get_arg(1);
        // let tbox  = $whatsit.get_arg(2);
        // if let Some(h) = GetKeyVal!(spec, "to") {
        //   whatsit.set_height(h);
        // } else if let Some(s) = GetKeyVal!(spec, "spread") {
        //   whatsit.set_height(tbox.get_height().add(s));
        // }
    }
  );

  DefConstructor!("\\vtop BoxSpecification VBoxContents", sub[document, args, _props] {
      let contents = args[1].as_ref().unwrap();
      insert_block(document, contents, string_map!("vattach" => "top"))?;
    },
    // sizer       => '#2',
    mode        => "text",
    after_digest => sub[_whatsit] {
      // TODO: Height arith
      //   my $spec = $whatsit.get_arg(1);
      //   my $box  = $whatsit.get_arg(2);
      //   if (my $h = GetKeyVal($spec, 'to')) {
      //     $whatsit->setHeight($h); }
      //   elsif (my $s = GetKeyVal($spec, 'spread')) {
      //     $whatsit->setHeight($box->getHeight->add($s)); }
      //   return; });
    }
  );
    
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Commands to store and use boxes
  // ----------------------------------------------------------------------
  // \setbox         c  assigns an hbox, vbox, or vtop to a box register.
  // \dp             iq is the depth of a box.
  // \ht             iq is the height of a box.
  // \wd             iq is the width of a box.
  // \box            c  puts the box's contents in the current list and empties the box.
  // \copy           c  puts the box's contents in the current list but does not empty the box     .
  // \unhbox         c  puts unwrapped hbox contents in the current list and empties the box.
  // \unhcopy        c  puts unwrapped hbox contents in the current list but does not empty the box.
  // \unvbox         c  puts unwrapped vbox contents in the current list and empties the box.
  // \unvcopy        c  puts unwrapped vbox contents in the current list but does not empty the box.
  // \lastbox        c  is void or the last hbox or vbox on the current list.
  // ======================================================================


  DefPrimitive!("\\lastbox", {// Hopefully, the correct box got seen!
    pop_box_list().map(|b| vec![b]).unwrap_or_default()
  });

  DefPrimitive!("\\setbox Number SkipMatch:=", sub[(number)] {
    // If there is any afterAssignment tokens, move them over so BoxContents parameter will use them
    if let Some(after_token) = state::remove_value("afterAssignment") {
      state::assign_value("BeforeNextBox", after_token, None);
    }
    // Save global flag, since we're digesting to get the box content, which resets the flag!
    // Should afterDigest be responsible for resetting flags?
    let scope = if get_prefix("global") {
      Some(Scope::Global)
    } else {
      None
    };
    clear_prefixes(); // before invoke, below; we've saved the only relevant one (global)
    let mut rest = if let Some(xtoken) = gullet::read_x_token(None, false)? {
        stomach::invoke_token(&xtoken)?
    } else { Vec::new() };
    let stuff = if !rest.is_empty() {
      Stored::Digested(rest.remove(0))
    } else {
      Stored::None
    };
    state::assign_value(&format!("box{}", number.value_of()), stuff, scope);
    rest
  });


  // # <box dimension> = \ht | \wd | \dp
  DefRegister!("\\ht Number", Dimension::new(0),
  getter => sub[args] {
    let n = args.remove(0).expect_number();
    with_value(&format!("box{}", n.value_of()), |val_opt|
    if let Some(Stored::Digested(thebox)) = val_opt {
      thebox.get_height()
    } else {
      Some(RegisterValue::Dimension(Dimension::default()))
    })},
  setter => sub[value,_scope,args] {
    let n = args.remove(0).expect_number();
    let boxkey = format!("box{}", n.value_of());
    with_value_mut(&boxkey, |val_opt|
    if let Some(Stored::Digested(thebox)) = val_opt {
      thebox.set_height(value);
    })});

  DefRegister!("\\wd Number", Dimension::default(),
  getter => sub[args] {
    let n = args.remove(0).expect_number();
    let boxid = format!("box{}", n.value_of());
    let mut stuff = checkout_value(&boxid);
    let result = {if let Some(Stored::Digested(ref mut thebox)) = stuff {
      match thebox.get_width(None) {
        Ok(v) => v,
        Err(e) => {
          let err = || {Error!("method", "get_width", format!("{e}")); Ok(()) };
          err().ok();
          None
        }
      }
    } else {
      Some(RegisterValue::Dimension(Dimension::default()))
    }};
    if let Some(thebox) = stuff {
      checkin_value(&boxid, thebox);
    }
    result
  },
  setter => sub[value,_scope,args] {
    let n = args.remove(0).expect_number();
    let boxkey = format!("box{}", n.value_of());
    with_value_mut(&boxkey, |val_opt|
    if let Some(Stored::Digested(thebox)) = val_opt {
      thebox.set_width(value);
    })});

  DefRegister!("\\dp Number", Dimension::new(0),
  getter => sub[args] {
    let n = args.remove(0).expect_number();
    with_value(&format!("box{}", n.value_of()),|val_opt|
      if let Some(Stored::Digested(thebox)) = val_opt {
        thebox.get_depth()
      } else {
        Some(RegisterValue::Dimension(Dimension::default()))
      })},
  setter => sub[value,_scope,args] {
    let n = args.remove(0).expect_number();
    let boxkey = format!("box{}", n.value_of());
    with_value_mut(&boxkey, |val_opt|
    if let Some(Stored::Digested(thebox)) = val_opt {
      thebox.set_depth(value);
    })
  });

  DefPrimitive!("\\box Number", sub[(number)] {
    let box_key = s!("box{}", number.value_of());
    if let Some(Stored::Digested(stuff)) = state::remove_value(&box_key) {
      Ok(vec![stuff])
    } else {
      Ok(Vec::new())
    }
  });

  DefPrimitive!("\\copy Number", sub[(number)] {
    let box_key = s!("box{}", number.value_of());
    if let Some(Stored::Digested(stuff)) = lookup_value(&box_key) {
      Ok(vec![stuff])
    } else {
      Ok(Vec::new())
    }
  });

  // \unhbox<8bit>, \unhcopy<8bit>
  // DefPrimitive('\unhbox Number', sub {
  //     my $box   = 'box' . $_[1]->valueOf;
  //     my $stuff = LookupValue($box);
  //     AssignValue($box, undef);
  //     (defined $stuff ? $stuff->unlist : ()); });
  // DefPrimitive('\unhcopy Number', sub {
  //     my $box   = 'box' . $_[1]->valueOf;
  //     my $stuff = LookupValue($box);
  //     (defined $stuff ? $stuff->unlist : ()); });
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

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Various box related parameters
  // ----------------------------------------------------------------------
  // \prevdepth      iq is the depth of the last box added to the current vertical list.
  // \boxmaxdepth    pd is the maximum possible depth of a vertical box.
  // \badness        iq is 0-10,000 and represents the badness of the glue settings
  //                    in the last constructed box.
  // \hbadness       pi is the badness above which bad hboxes are reported.
  // \vbadness       pi is the badness above which bad vboxes are reported.
  // \hfuzz          pd is the overrun allowed before overfull hboxes are reported.
  // \vfuzz          pd is the overrun allowed before overfull vboxes are reported.
  // \overfullrule   pd is the width of the rule appended to an overfull box.
  // ======================================================================
  DefRegister!("\\prevdepth", Dimension::new(0));
  DefRegister!("\\boxmaxdepth", Dimension!("16383.99999pt"));
  DefRegister!("\\hfuzz", Dimension!("0.1pt"));
  DefRegister!("\\vfuzz", Dimension!("0.1pt"));
  DefRegister!("\\overfullrule", Dimension!("5pt"));
  DefRegister!("\\badness",Number!(0), readonly => true);
  DefRegister!("\\hbadness", Number!(1000), readonly => true);
  DefRegister!("\\vbadness", Number!(1000));

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Rules and Leaders
  // ----------------------------------------------------------------------
  // \hrule          c  makes a rule box in vertical mode.
  // \vrule          c  makes a rule box in horizontal mode.
  // \cleaders       c  insert centered leaders.
  // \leaders        c  fill space using specified glue with a box or rule.
  // \xleaders       c  insert expanded leaders.
  // ======================================================================
  DefParameterType!(RuleSpecification, sub[_inner, _extra] {
    let mut keyvals = KeyVals::new(
      KeyvalsConfig{ skip_missing: keyvals::SkipMissing::All, .. KeyvalsConfig::default()});
    while let Some(key) = gullet::read_keyword(&["width", "height", "depth"])? {
      keyvals.set_value(&key, ArgWrap::Dimension(gullet::read_dimension()?), false)?;
    }
    keyvals
  },
  optional => true,
  predigest => sub[arg] { Ok(arg.undigested()) });
  
  // \hrule, \vrule are awkward in trying to deal with 3 cases
  //  * as rules within an alignment/table
  //  * as separating lines within text
  //  * as graphical lines within svg
  // and each has different requirements for size
  DefConstructor!("\\vrule RuleSpecification","",
  // "?#invisible()(?#isVerticalRule()\
  //   (<ltx:rule height='&GetKeyVal(#1,height)' depth='&GetKeyVal(#1,depth)' \
  //    width='&GetKeyVal(#1,width)' color='#color'/>))",
  after_digest => sub [whatsit] {
  //   my $dims   = $whatsit->getArg(1);
  //   my $width  = GetKeyVal($dims, 'width');
  //   my $height = GetKeyVal($dims, 'height');
  //   my $depth  = GetKeyVal($dims, 'depth');
  //   $whatsit->setProperty(width  => $width)  if $width;
  //   $whatsit->setProperty(height => $height) if $height;
  //   $whatsit->setProperty(depth  => $depth)  if $depth;
  //   my $w = ($width  ? $width->ptValue  : undef);
  //   my $h = ($height ? $height->ptValue : undef);
  //   my $d = ($depth  ? $depth->ptValue  : undef);
  //   $h -= $d if $h && $d;    # - ??

    if let Some(_alignment) = lookup_alignment() {
  //     if (((!defined $h) && (!defined $w)) || ((defined $h) && ($h > 20))
  //       || ((defined $h) && (defined $w) && ($h > 3 * $w))) {
  // This isXxxxRule property is to determine if it is used for separating rules within alignments
      whatsit.set_property("isVerticalRule", true);
    }
  // }
  //   elsif ((defined $w) && ($w == 0)) {
  //     $whatsit->setProperty(invisible => 1); }
  //   else {
  //     $dims->setValue(width => '1px') unless defined $w; }
  //   if (my $color = LookupValue('font')->getColor) {
  //     if ($color ne 'black') {
  //       $whatsit->setProperty(color => $color); } }
    Ok(Vec::new())
  });

  DefConstructor!("\\hrule RuleSpecification","",
  // "?#isHorizontalRule()\
  //   (<ltx:rule height='&GetKeyVal(#1,height)' depth='&GetKeyVal(#1,depth)'\
  //    width='&GetKeyVal(#1,width)' color='#color'/>)",
  after_digest=> { unported!() });
  // afterDigest => sub {
  //   my ($stomach, $whatsit) = @_;
  //   my $dims   = $whatsit->getArg(1);
  //   my $width  = GetKeyVal($dims, 'width');
  //   my $height = GetKeyVal($dims, 'height');
  //   my $depth  = GetKeyVal($dims, 'depth');
  //   $whatsit->setProperty(width  => $width)  if $width;
  //   $whatsit->setProperty(height => $height) if $height;
  //   $whatsit->setProperty(depth  => $depth)  if $depth;
  //   my $w = ($width  ? $width->ptValue  : undef);
  //   my $h = ($height ? $height->ptValue : undef);
  //   my $d = ($depth  ? $depth->ptValue  : undef);
  //   $h -= $d if $h && $d;    # - ??

  //   if (my $alignment = LookupValue('Alignment')) {
  //     # What is the intended logic here?
  //     if (((!defined $h) && (!defined $w)) || ((defined $w) && ($w > 20))
  //       || ((defined $h) && (defined $w) && ($w > 3 * $h))) {
  //       # This isXxxxRule property is to determine if it is used for separating rules within
  // alignments       $alignment->addLine('t');
  //       $whatsit->setProperty(isHorizontalRule => 1) } }
  //   else {
  //     $dims->setValue(width  => '100%') unless defined $w;
  //     $dims->setValue(height => '1px')  unless defined $h; }
  //   if (my $color = LookupValue('font')->getColor) {
  //     if ($color ne 'black') {
  //       $whatsit->setProperty(color => $color); } }
  //   return; });

  // Various leaders, ignored for now...
  DefPrimitive!("\\leaders", None);
  DefPrimitive!("\\cleaders", None);
  DefPrimitive!("\\xleaders", None);
});

// Risky: I think this needs to be digested as a body to work like TeX (?)
// but parameter think's it's just parsing from gullet...
pub fn read_box_contents(
  everybox_opt: Option<Tokens>,
) -> Result<Tokens> {
  while let Some(t) = gullet::read_token()? {
    if t.get_catcode() == Catcode::BEGIN {
      break;
    } // Skip till { or \bgroup
  }
  // Now, insert some extra tokens, if any, possibly from \afterassignment
  match state::remove_value("BeforeNextBox") {
    Some(Stored::Tokens(tokens)) => gullet::unread(tokens),
    Some(Stored::Token(token)) => gullet::unread_one(token),
    None | Some(Stored::None) => {},
    Some(other) => panic!("afterAssignment should be a token, got: {}", other),
  };
  // AND, insert any extra tokens passed in, due to everyhbox or everyvbox
  if let Some(everybox) = everybox_opt {
    gullet::unread(everybox);
  }
  Ok(Tokens!())
}
