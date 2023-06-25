use crate::package::*;

LoadDefinitions!({
  // <box> = \box <8bit> | \copy <8bit> | \lastbox | \vsplit <8bit> to <dimen>
  //   | \hbox <box specification>{<horizontal mode material>}
  //   | \vbox <box specification>{<vertical mode material>}
  //   | \vtop <box specification>{<vertical mode material>}
  // <box specification> = to <dimen><filler> | spread <dimen><filler> | <filler>

  // \setbox<number>=\hbox to <dimen>{<horizontal mode material>}

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

  DefPrimitive!("\\vsplit Number Match:to Dimension", sub[(number,_to,dimension)] {
    // analog to \box for now.
    let box_key   = s!("box{}", number.value_of());
    if let Some(Stored::Digested(stuff)) = lookup_value(&box_key) {
      adjust_box_color(&stuff)?;
      if stuff.is_empty()? { Digested::from(List::default()) } else { stuff }
    } else {
      Digested::from(List::default())
    }
  });


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
          KeyvalsConfig{skip_missing: true, ..KeyvalsConfig::default()});
        let dim = gullet::read_dimension()?;
        keyvals.set_value(&key.owned_tokens().unwrap().to_string(), dim.into(), false);
        keyvals.into()
      } else {
        Ok(None)
      }
    },
    optional => true);

  DefParameterType!(HBoxContents, sub[_inner, _extra] {
      read_box_contents(state::lookup_tokens("\\everyhbox"))
    },
    predigest => sub[arg] {
      predigest_box_contents(arg)
    });

  DefParameterType!(VBoxContents, sub[_inner, _extra] {
      read_box_contents(state::lookup_tokens("\\everyvbox"))
    },
    predigest => sub[arg] {
      predigest_box_contents( arg)
    });

  // This re-binds a number of important control sequences to their default text binding.
  // This is useful within common boxing or footnote macros that can appear within
  // alignments or special environments that have redefined many of these.
  AssignValue!("TEXT_MODE_BINDINGS"  => Stored::VecDequeStored(VecDeque::new()));
  AssignValue!("HTEXT_MODE_BINDINGS" => Stored::VecDequeStored(VecDeque::new()));
  AssignValue!("VTEXT_MODE_BINDINGS" => Stored::VecDequeStored(VecDeque::new()));
  PushValue!("HTEXT_MODE_BINDINGS" => Tokens!(T_MATH!(), T_CS!("\\@dollar@in@textmode")));
  PushValue!("VTEXT_MODE_BINDINGS" => Tokens!(T_MATH!(), T_CS!("\\@dollar@in@normalmode")));

  // TODO: collapseSVGGroup
  Tag!("svg:g", after_close => sub[_document, _node] {
    unimplemented!();
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
        if let Some(w) = GetKeyVal!(spec, "to") {
          w.into()
        } else if let Some(s) = GetKeyVal!(spec, "spread") {
          let s_num_opt : Option<RegisterValue> = s.into();
          let s_num = s_num_opt.unwrap_or_default();
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
    }
  );

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

  DefParameterType!(RuleSpecification, sub[_inner, _extra] {
      let mut keyvals = KeyVals::new(
        KeyvalsConfig{ skip_missing: true, .. KeyvalsConfig::default()});
      while let Some(key) = gullet::read_keyword(&["width", "height", "depth"])? {
        keyvals.set_value(&key, Stored::Dimension(gullet::read_dimension()?), false);
      }
      keyvals
    },
    optional => true,
    predigest => sub[arg] { Ok(arg.undigested()) }
  );

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
  after_digest=> {unimplemented!(); Ok(Vec::new())});
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
});


pub fn adjust_box_color(tbox: &Digested) -> Result<()> {
  let color_opt = lookup_font().and_then(|f| f.get_color()
    .map(|c| c.clone().into_owned()));
  if let Some(color) = color_opt {
    if color != "black" {
       adjust_box_color_rec(&color, HashMap::default(), tbox);
    }
  }
  Ok(())
}

fn adjust_box_color_rec(_color: &str, _props: HashMap<String,String>, _tbox: &Digested) {
  unimplemented!();
}