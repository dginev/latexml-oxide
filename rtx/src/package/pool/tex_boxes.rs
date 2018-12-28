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

fn predigest_box_contents(stomach: &mut Stomach, _tokens: Tokens, state: &mut State) -> Result<Digested> {
  let mut contents = stomach.invoke_token(&T_BEGIN!(), state)?;
  Ok(contents.remove(0))
}

pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

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
      let key = gullet.read_keyword(&["to", "spread"], state)?;
      // TODO
      // let keyvals = KeyVals::new(None, None, skipMissing => 1);
      // keyvals.set_value(key, gullet.read_dimension);
      // keyvals
      Ok(Tokens!())
    }), optional => true);

  DefParameterType!("HBoxContents", reader => reader!(gullet, inner, extra, state, {
      read_box_contents(gullet, state.lookup_tokens("\\everyhbox"), state)
    }),
    reader_predigest=>reader_predigest!(stomach, arg, state, {
      predigest_box_contents(stomach, arg, state)
    })
  ); // Cause it already is digested!

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

  // sub reenterTextMode {
  //   my ($verticalmode) = @_;
  //   map { Let($$_[0], $$_[1]) }
  //     @{ LookupValue(($verticalmode ? 'VTEXT_MODE_BINDINGS' : 'HTEXT_MODE_BINDINGS')) },
  //     @{ LookupValue('TEXT_MODE_BINDINGS') };
  //   return }

  // sub REF {
  //   my ($thing, $key) = @_;
  //   return $thing && $$thing{$key}; }

  DefConstructor!("\\hbox BoxSpecification HBoxContents", sub[document, args, props, state] {
    // "<ltx:text width='#width' _noautoclose='1'>#2</ltx:text>",
    unpack!(args => spec, contents);
  //     my ($document, $spec, $contents, %props) = @_;
  //     my $model   = $document->getModel;
  //     my $context = $document->getElement;
  //     my $current = $context;

  //     # What is the CORRECT (& general) way to ask whether we're in "vertical mode"??
  //     #  my $vmode = $tag eq 'ltx:inline-block'; # ie, explicitly \vbox !?!?!?!
  //     my $vmode = $current && $current->getAttribute('_vertical_mode_');
  //     my $newtag = ($vmode ? 'ltx:p' : 'ltx:text');
  //     my $node = $document->openElement($newtag, _noautoclose => 1,
  //       width => $props{width});
  //     $document->absorb($contents);
  //     $document->closeNode($node); },

  //   mode => 'text', bounded => 1,
  //   sizer => '#2',
  //   # Workaround for $ in alignment; an explicit \hbox gives us a normal $.
  //   # And also things like \centerline that will end up bumping up to block level!
  //   beforeDigest => sub { reenterTextMode(); },

  //   afterDigest => sub {
  //     my ($stomach, $whatsit) = @_;
  //     my $spec = $whatsit->getArg(1);
  //     my $box  = $whatsit->getArg(2);
  //     if (my $w = GetKeyVal($spec, 'to')) {
  //       $whatsit->setWidth($w); }
  //     elsif (my $s = GetKeyVal($spec, 'spread')) {
  //       $whatsit->setWidth($box->getWidth->add($s)); }

  });

  Ok(())
}
