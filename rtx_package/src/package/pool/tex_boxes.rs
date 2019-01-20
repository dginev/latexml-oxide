use crate::package::*;

// Risky: I think this needs to be digested as a body to work like TeX (?)
// but parameter think's it's just parsing from gullet...
fn read_box_contents(gullet: &mut Gullet, everybox_opt: Option<Tokens>, state: &mut State) -> Result<Tokens> {
  while let Some(t) = gullet.read_token(state) {
    if t == T_BEGIN!() {
      break;
    } // Skip till { or \bgroup
  }
  // Now, insert some extra tokens, if any, possibly from \afterassignment
  if let Some(ref token) = state.lookup_tokens("BeforeNextBox") {
    state.assign_value("BeforeNextBox", None, Some(Scope::Global));
    gullet.unread(token);
  }
  // AND, insert any extra tokens passed in, due to everyhbox or everyvbox
  if let Some(everybox) = everybox_opt {
    gullet.unread(&everybox);
  }
  Ok(Tokens!())
}

fn predigest_box_contents(stomach: &mut Stomach, _tokens: Tokens, state: &mut State) -> Result<Option<Digested>> {
  let mut contents = stomach.invoke_token(&T_BEGIN!(), state)?;
  Ok(Some(contents.remove(0)))
}

LoadDefinitions!(state, {
  // <box> = \box <8bit> | \copy <8bit> | \lastbox | \vsplit <8bit> to <dimen>
  //   | \hbox <box specification>{<horizontal mode material>}
  //   | \vbox <box specification>{<vertical mode material>}
  //   | \vtop <box specification>{<vertical mode material>}
  // <box specification> = to <dimen><filler> | spread <dimen><filler> | <filler>

  // \setbox<number>=\hbox to <dimen>{<horizontal mode material>}

  // DefPrimitive('\setbox Number SkipMatch:=', sub {
  //     my ($stomach) = @_;
  //     no warnings 'recursion';
  //     my $box = 'box' . $_[1]->valueOf;
  //     # If there is any afterAssignment tokens, move them over so BoxContents parameter will use
  // them     if (my $token = LookupValue('afterAssignment')) {
  //       AssignValue('afterAssignment' => undef, 'global');
  //       AssignValue('BeforeNextBox' => $token); }
  //     # Save global flag, since we're digesting to get the box content, which resets the flag!
  //     # Should afterDigest be responsible for resetting flags?
  //     my $scope = $STATE->getPrefix('global') && 'global';
  //     $STATE->clearPrefixes;    # before invoke, below; we've saved the only relevant one
  // (global)     my ($stuff, @rest) = $stomach->invokeToken($stomach->getGullet->readXToken);
  //     AssignValue('box' . $_[1]->valueOf => $stuff, $scope);
  //     @rest; });

  // DefPrimitive('\box Number', sub {
  //     my $box   = 'box' . $_[1]->valueOf;
  //     my $stuff = LookupValue($box);
  //     AssignValue($box, undef);
  //     ($stuff ? $stuff->unlist : ()); });

  // DefPrimitive('\copy Number', sub {
  //     my $box   = 'box' . $_[1]->valueOf;
  //     my $stuff = LookupValue($box);
  //     ($stuff ? $stuff->unlist : ()); });

  // sub revert_spec {
  //   my ($whatsit, $keyword) = @_;
  //   my $value = $whatsit->getProperty($keyword);
  //   return ($value ? (Explode($keyword), Revert($value)) : ()); }

  DefParameterType!("BoxSpecification",  reader => reader!(gullet, inner, extra, state, {
      if let Some(key) = gullet.read_keyword(&["to", "spread"], state)? {
        Ok(key)
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
        let mut keyvals = KeyVals::new(None, None, map!("skipMissing" => true), state);
        let dim = stomach.get_gullet_mut().read_dimension(state)?;
        keyvals.set_value(&key.to_string(), dim.into(), false, state);
        keyvals.into()
      } else {
        Ok(None)
      }
    }),
    optional => true);

  DefParameterType!("HBoxContents", reader => reader!(gullet, inner, extra, state, {
      read_box_contents(gullet, state.lookup_tokens("\\everyhbox"), state)
    }),
    reader_predigest=>reader_predigest!(stomach, arg, state, { predigest_box_contents(stomach, arg, state) })
  );

  // DefParameterType('VBoxContents', sub {
  //     read_box_contents($_[0], LookupValue('\everyvbox')); },
  //   undigested => 1);    # Cause it already is digested!

  // # DefParameterType('BoxContents',sub {
  // #   my($gullet)=@_;
  // #   my $t;
  // #   while(($t=$gullet->readToken) && !Equals($t,T_BEGIN)){} # Skip till { or \bgroup
  // #   my($contents,@stuff) = $STATE->getStomach->invokeToken(T_BEGIN);
  // #   $contents; },
  // #        undigested=>1); # Cause it already is digested!

  // # This re-binds a number of important control sequences to their default text binding.
  // # This is useful within common boxing or footnote macros that can appear within
  // # alignments or special environments that have redefined many of these.
  // AssignValue(TEXT_MODE_BINDINGS  => []);
  // AssignValue(HTEXT_MODE_BINDINGS => []);
  // AssignValue(VTEXT_MODE_BINDINGS => []);
  // PushValue(HTEXT_MODE_BINDINGS => [T_MATH, T_CS('\@dollar@in@textmode')]);
  // PushValue(VTEXT_MODE_BINDINGS => [T_MATH, T_CS('\@dollar@in@normalmode')]);
  // ###PushValue(TEXT_MODE_BINDINGS => [T_CS('\centerline'), T_CS('\relax')]);

  // sub REF {
  //   my ($thing, $key) = @_;
  //   return $thing && $$thing{$key}; }

  DefConstructor!("\\hbox BoxSpecification HBoxContents", sub[document, args, props, state] {
      // "<ltx:text width='#width' _noautoclose='1'>#2</ltx:text>",
      unpack!(args => spec, contents);
      let current = document.get_element();

      // What is the CORRECT (& general) way to ask whether we're in "vertical mode"??
      //  my $vmode = $tag eq 'ltx:inline-block'; # ie, explicitly \vbox !?!?!?!
      let vmode = match current {
        None => false,
        Some(node) => node.get_attribute("_vertical_mode_").is_some()
      };
      let newtag = if vmode { "ltx:p" } else { "ltx:text" };
      let width : String = if let Some(Stored::Dimension(ref w)) = props.get("width") {
        w.to_attribute()
      } else {
        String::new()
      };
      let node = document.open_element(newtag, Some(string_map!("_noautoclose" => "true", "width" => width)), None, state)?;
      document.absorb(contents,state)?;
      document.close_node(&node, state)?;
      debug!("FINAL DOC: {:?}", document.document.node_to_string(&document.get_element().unwrap()));
    },
    mode => "text".into_option(),
    bounded => true,
    // sizer => "#2",
    //   # Workaround for $ in alignment; an explicit \hbox gives us a normal $.
    //   # And also things like \centerline that will end up bumping up to block level!
    before_digest => beforeproc!(stomach, state, {reenter_text_mode(false, state)}),
    after_digest => afterproc!(stomach, whatsit, state, {
      let mut width : Option<RegisterValue>= None;
      {
        let spec = whatsit.get_arg(1);
        let tbox = whatsit.get_arg(2).unwrap();
        if let Some(w) = GetKeyVal!(spec, "to") {
          width = w.into();
        } else if let Some(s) = GetKeyVal!(spec, "spread") {
          let s_num_opt : Option<RegisterValue> = s.into();
          let s_num = s_num_opt.unwrap_or_else(|| Number::new(0.0).into());
          width = Some( tbox.get_width(state).unwrap().add(s_num) );
        }
      }
      if let Some(w) = width {
        whatsit.set_width(w);
      }

    })
  );
});
