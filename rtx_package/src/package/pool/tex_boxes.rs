use crate::package::*;

LoadDefinitions!(state, {
  // <box> = \box <8bit> | \copy <8bit> | \lastbox | \vsplit <8bit> to <dimen>
  //   | \hbox <box specification>{<horizontal mode material>}
  //   | \vbox <box specification>{<vertical mode material>}
  //   | \vtop <box specification>{<vertical mode material>}
  // <box specification> = to <dimen><filler> | spread <dimen><filler> | <filler>

  // \setbox<number>=\hbox to <dimen>{<horizontal mode material>}

  DefPrimitive!("\\setbox Number SkipMatch:=", sub[stomach, (number), state] {
    // If there is any afterAssignment tokens, move them over so BoxContents parameter will use them
    if let Some(after_token) = state.remove_value("afterAssignment") {
      state.assign_value("BeforeNextBox", after_token, None);
    }
    // Save global flag, since we're digesting to get the box content, which resets the flag!
    // Should afterDigest be responsible for resetting flags?
    let scope = if state.get_prefix("global") {
      Some(Scope::Global)
    } else {
      None
    };
    state.clear_prefixes(); // before invoke, below; we've saved the only relevant one (global)
    let mut rest = if let Some(xtoken) = stomach.get_gullet_mut().read_x_token(None, false, state)? {
      stomach.invoke_token(&xtoken, state)?
    } else { Vec::new() };
    let stuff = if !rest.is_empty() {
      Stored::Digested(Box::new(rest.remove(0)))
    } else {
      Stored::None
    };
    state.assign_value(&format!("box{}", number.value_of()), stuff, scope);
    rest
  });

  DefPrimitive!("\\box Number", sub[gullet, (number), state] {
    let box_key = s!("box{}", number.value_of());
    if let Some(Stored::Digested(stuff)) = state.remove_value(&box_key) {
      Ok(vec![*stuff])
    } else {
      Ok(Vec::new())
    }
  });

  DefPrimitive!("\\copy Number", sub[stomach, (number), state] {
    let box_key = s!("box{}", number.value_of());
    if let Some(Stored::Digested(stuff)) = state.lookup_value(&box_key) {
      let cloned_stuff : Digested = (**stuff).clone();
      Ok(vec![cloned_stuff])
    } else {
      Ok(Vec::new())
    }
  });

  DefParameterType!(BoxSpecification,  reader => reader!(gullet, inner, extra, state, {
      if let Some(key) = gullet.read_keyword(&["to", "spread"], state)? {
        Ok(Tokens!(T_OTHER!(key)))
      } else {
        Ok(Tokens!())
      }
    }),
    // The predigest closure is new for rtx, as it was a single closure in the latexml implementation
    // The key problem is that in rtx the parameter type interfaces are well-typed, so it is not possible
    // to remain elegant while at the same time have access to the stomach AND digest.
    // Hence, the `reader` is exclusively responsible for using the gullet to obtain tokens,
    // while early/immediate digestion via the stomach can be achieved by using the separate `reader_predigest` interface
    // Importantly, reader_predigest forces the parameter to be usable only for stomach-capable bindings,
    // namely DefConstructor, DefPrimitive or DefEnvironment
    reader_predigest => reader_predigest!(stomach, key, state, {
      if !key.is_empty() {
        let mut keyvals = KeyVals::new(KeyValsOptions{skip_missing: true, ..KeyValsOptions::default()}, state);
        let dim = stomach.get_gullet_mut().read_dimension(state)?;
        keyvals.set_value(&key.owned_tokens().unwrap().to_string(), dim.into(), false, state);
        keyvals.into()
      } else {
        Ok(None)
      }
    }),
    optional => true);

  DefParameterType!(HBoxContents, reader => reader!(gullet, _inner, _extra, state, {
      read_box_contents(gullet, state.lookup_tokens("\\everyhbox"), state)
    }),
    reader_predigest=>reader_predigest!(stomach, arg, state, { predigest_box_contents(stomach, arg, state) })
  );

  DefParameterType!(VBoxContents, reader=>reader!(gullet, _inner, _extra, state, {
      read_box_contents(gullet, state.lookup_tokens("\\everyvbox"), state)
    }),
    reader_predigest=>reader_predigest!(stomach, arg, state, { predigest_box_contents(stomach, arg, state) })
  );

  // This re-binds a number of important control sequences to their default text binding.
  // This is useful within common boxing or footnote macros that can appear within
  // alignments or special environments that have redefined many of these.
  AssignValue!("TEXT_MODE_BINDINGS"  => Stored::VecDequeStored(VecDeque::new()));
  AssignValue!("HTEXT_MODE_BINDINGS" => Stored::VecDequeStored(VecDeque::new()));
  AssignValue!("VTEXT_MODE_BINDINGS" => Stored::VecDequeStored(VecDeque::new()));
  PushValue!("HTEXT_MODE_BINDINGS" => Tokens!(T_MATH!(), T_CS!("\\@dollar@in@textmode")));
  PushValue!("VTEXT_MODE_BINDINGS" => Tokens!(T_MATH!(), T_CS!("\\@dollar@in@normalmode")));

  DefConstructor!("\\hbox BoxSpecification HBoxContents", sub[document, args, props, state] {
      // "<ltx:text width='#width' _noautoclose='1'>#2</ltx:text>",
      unpack_opt_ref!(args => spec_opt, contents_opt);
      let contents = contents_opt.unwrap().as_ref().unwrap();
      let current_opt = document.get_element();

      // What is the CORRECT (& general) way to ask whether we're in "vertical mode"??
      //  my $vmode = $tag eq 'ltx:inline-block'; # ie, explicitly \vbox !?!?!?!
      let is_svg  = if let Some(ref current) = current_opt {
        document.get_node_qname(current, state).starts_with("svg:")
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
      let node = document.open_element(newtag, Some(string_map!("_noautoclose" => "true", "width" => width)), None, state)?;
      // Note on the clone: Remember that contents is a Digested, i.e. we are cloning an Arc<> wrapper, which is relatively cheap.
      // see the documentation on `Digested` on why we don't have a neater way of dealing with this.
      document.absorb(contents, None, state)?;
      if !is_svg {
        while !document.get_element().unwrap().has_attribute("_beginscope") &&
          document.maybe_close_element("svg:g", state)?.is_some() {}
        document.maybe_close_element("svg:svg", state)?;
        document.maybe_close_node(&node, state)?;
      } else {
        document.maybe_close_element("svg:g", state)?;
      }
    },
    mode => "text",
    bounded => true,
    sizer => "#2",
    //   # Workaround for $ in alignment; an explicit \hbox gives us a normal $.
    //   # And also things like \centerline that will end up bumping up to block level!
    before_digest => sub[stomach, state] {reenter_text_mode(false, stomach.get_gullet_mut(), state)},
    after_digest => sub[stomach, whatsit, state] {
      let width : Option<RegisterValue> = {
        let spec = whatsit.get_arg(1);
        if let Some(w) = GetKeyVal!(spec, "to") {
          w.into()
        } else if let Some(s) = GetKeyVal!(spec, "spread") {
          let s_num_opt : Option<RegisterValue> = s.into();
          let s_num = s_num_opt.unwrap_or_default();
          let mut tbox = whatsit.get_arg_mut(2).unwrap();
          let current_w = tbox.get_width(None, state)?.unwrap();
          let new_w = current_w.add(s_num);
          Some( new_w )
        } else {
          None
        }
      };
      if let Some(w) = width {
        whatsit.set_width(w);
      }
    }
  );

  DefConstructor!("\\vbox BoxSpecification VBoxContents", sub[document, args, props, state] {
      let contents = args[1].as_ref().unwrap();
      let block = insert_block(document, contents, string_map!("vattach" => "bottom"), state);
    },
    // sizer       => "#2",
    mode        => "text",
    after_digest => sub[stomach, whatsit, state] {
      // TODO: Height arith
        // let spec = whatsit.get_arg(1);
        // let tbox  = $whatsit.get_arg(2);
        // if let Some(h) = GetKeyVal!(spec, "to") {
        //   whatsit.set_height(h);
        // } else if let Some(s) = GetKeyVal!(spec, "spread") {
        //   whatsit.set_height(tbox.get_height().add(s));
        // }
        ()
    }
  );

  DefConstructor!("\\vtop BoxSpecification VBoxContents", sub[document, args, props, state] {
      let contents = args[1].as_ref().unwrap();
      insert_block(document, contents, string_map!("vattach" => "top"), state)?;
    },
    // sizer       => '#2',
    mode        => "text",
    after_digest => sub[stomach, whatsit, state] {
      // TODO: Height arith
      //   my $spec = $whatsit.get_arg(1);
      //   my $box  = $whatsit.get_arg(2);
      //   if (my $h = GetKeyVal($spec, 'to')) {
      //     $whatsit->setHeight($h); }
      //   elsif (my $s = GetKeyVal($spec, 'spread')) {
      //     $whatsit->setHeight($box->getHeight->add($s)); }
      //   return; });
      ()
    }
  );

  DefParameterType!(RuleSpecification, reader=>reader!(gullet, inner, extra, state, {
    unimplemented!(); ()
    // my $keyvals = LaTeXML::Core::KeyVals->new(undef, undef, skipMissing => 1);
    // while (my $key = $gullet->readKeyword('width', 'height', 'depth')) {
    //   $keyvals->setValue($key, $gullet->readDimension); }
    // $keyvals;
    }),
    optional => true,
    reader_predigest => undigested!()
  );

  DefConstructor!("\\vrule RuleSpecification","",
  // "?#invisible()(?#isVerticalRule()\
  //   (<ltx:rule height='&GetKeyVal(#1,height)' depth='&GetKeyVal(#1,depth)' \
  //    width='&GetKeyVal(#1,width)' color='#color'/>))",
  after_digest=> {unimplemented!(); ()});
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
  //     if (((!defined $h) && (!defined $w)) || ((defined $h) && ($h > 20))
  //       || ((defined $h) && (defined $w) && ($h > 3 * $w))) {
  //       # This isXxxxRule property is to determine if it is used for separating rules within alignments
  //       $whatsit->setProperty(isVerticalRule => 1) } }
  //   elsif ((defined $w) && ($w == 0)) {
  //     $whatsit->setProperty(invisible => 1); }
  //   else {
  //     $dims->setValue(width => '1px') unless defined $w; }
  //   if (my $color = LookupValue('font')->getColor) {
  //     if ($color ne 'black') {
  //       $whatsit->setProperty(color => $color); } }
  //   return; }

  DefConstructor!("\\hrule RuleSpecification","",
  // "?#isHorizontalRule()\
  //   (<ltx:rule height='&GetKeyVal(#1,height)' depth='&GetKeyVal(#1,depth)'\
  //    width='&GetKeyVal(#1,width)' color='#color'/>)",
  after_digest=> {unimplemented!(); ()});
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
  //       # This isXxxxRule property is to determine if it is used for separating rules within alignments
  //       $alignment->addLine('t');
  //       $whatsit->setProperty(isHorizontalRule => 1) } }
  //   else {
  //     $dims->setValue(width  => '100%') unless defined $w;
  //     $dims->setValue(height => '1px')  unless defined $h; }
  //   if (my $color = LookupValue('font')->getColor) {
  //     if ($color ne 'black') {
  //       $whatsit->setProperty(color => $color); } }
  //   return; });
});
