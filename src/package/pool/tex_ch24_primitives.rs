use rtx_core::TexMode;
use rtx_core::tbox::Tbox;
use rtx_core::list::List;
use package::*;

pub fn load_definitions(outer_state: &mut State) -> Result<()> {
  SetupBindingMacros!(outer_state);

  //======================================================================
  // Remaining Mode independent primitives in Ch.24, pp.279-280
  // \relax was done as expandable (isn't that right?)
  // }
  // Note, we don't bother making sure begingroup is ended by endgroup.

  // These define the handler for { } (or anything of catcode BEGIN, END)

  // These are actually TeX primitives, but we treat them as a Whatsit so they
  // remain in the constructed tree.
  DefPrimitiveI!(
    "{",
    primitivesub!(stomach, _args, state, {
      stomach.bgroup(state);
      let open = Tbox::new(
        String::new(),
        None,
        None,
        Tokens!(T_BEGIN!()),
        HashMap::new(),
        state,
      );
      let ismath = state.lookup_bool("IN_MATH");
      let mode = if ismath { TexMode::Math } else { TexMode::Text };
      let body = try!(stomach.digest_next_body(false, state));
      let mut boxes = vec![Digested::Box(open)];
      boxes.extend(body);
      let return_list = List {
        boxes: boxes,
        mode: Some(mode),
        font: None,
      };

      Ok(vec![Digested::List(return_list)])
    })
  );

  DefPrimitiveI!(
    "}",
    primitivesub!(stomach, _args, state, {
      let f = state.lookup_font();
      try!(stomach.egroup(state));
      let return_box = Tbox::new(
        String::new(),
        f,
        None,
        Tokens!(T_END!()),
        HashMap::new(),
        state,
      );
      Ok(vec![Digested::Box(return_box)])
    })
  );

  // // These are for those screwy cases where you need to create a group like box,
  // // more than just bgroup, egroup,
  // // BUT you DON'T want extra {, } showing up in any untex-ing.
  // DefConstructor('\@hidden@bgroup', '//body', beforeDigest => sub { $_[0]->bgroup; },
  // captureBody => 1,   reversion => sub { Revert($_[0]->getProperty('body')); });
  // DefConstructor('\@hidden@egroup', '', afterDigest => sub { $_[0]->egroup; },
  //   reversion => '');

  DefPrimitiveI!(
    "\\begingroup",
    primitiveproc!(stomach, _args, state, {
      stomach.begingroup(state);
    })
  );
  DefPrimitiveI!(
    "\\endgroup",
    primitiveproc!(stomach, _args, state, {
      try!(stomach.endgroup(state));
    })
  );

  // // Debugging aids; Ignored!
  // DefPrimitive('\show Token',     undef);
  // DefPrimitive('\showbox Number', undef);
  // DefPrimitive('\showlists',      undef);
  // DefPrimitive('\showthe Token',  undef);

  // // DefPrimitive('\shipout ??
  DefPrimitiveI!("\\ignorespaces SkipSpaces", noprimitive!());

  // // \afterassignment saves ONE token (globally!) to execute after the next assignment
  // DefPrimitive('\afterassignment Token', sub { AssignValue(afterAssignment => $_[1], 'global');
  // }); // \aftergroup saves ALL tokens (from repeated calls) to be executed IN ORDER after the
  // next egroup or } DefPrimitive('\aftergroup Token', sub { PushValue(afterGroup => $_[1]); });

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

  Ok(())
}
