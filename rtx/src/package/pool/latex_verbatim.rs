use crate::package::*;

//**********************************************************************
// C.6.4 Verbatim
//**********************************************************************
pub fn load_definitions(outer_state: &mut State) -> Result<()> {
  SetupBindingMacros!(outer_state);

  // NOTE: how's the best way to get verbatim material through?
  DefEnvironment!("{verbatim}", "<ltx:verbatim>#body</ltx:verbatim>");
  DefEnvironment!("{verbatim*}", "<ltx:verbatim>#body</ltx:verbatim>");
  Let!("\\@verbatim", "\\verbatim");
  // Close enough?
  // verbatim is a bit of special case;
  // It looks like an environment, but it only ends with an explicit "\end{verbatim}" on it's own
  // line. So, we'll end up doing things more manually.
  // We're going to sidestep the Gullet for inputting,
  // and also the usual environment capture.

  DefConstructor!(cs["\\begin{verbatim}"], None, "<ltx:verbatim font='#font'>#body</ltx:verbatim>",
    before_digest => beforesub!(stomach, state, {
      stomach.bgroup(state);
      let mut stuff = Vec::new();
      if let Some(b) = state.lookup_tokens("@environment@verbatim@atbegin") {
        stuff.push(stomach.digest(b.unlist(), state)?);
      }
      state.assign_value("current_environment", "verbatim", None);
      DefMacro!("\\@currenvir", "verbatim", state);
      MergeFont!(family => "typewriter", state);
      Ok(stuff)
    }),
    after_digest => afterproc!(stomach, whatsit, state, {
    //       my $font   = $whatsit->getFont;
    //       my $loc    = $whatsit->getLocator;
    //       my $end    = "\\end{verbatim}";
    //       my @lines  = ();
    //       my $gullet = $stomach->getGullet;
    //       while (defined(my $line = $gullet->readRawLine)) {
    //         // The raw chars will still have to be decoded (but not space!!)
    //         $line = join('', map { ($_ eq ' ' ? ' ' : FontDecodeString($_, 'OT1_typewriter')) }
    //             split(//, $line));
    //         if ($line =~ /^(.*?)\\end\{verbatim\}(.*?)$/) {
    //           push(@lines, $1 . "\n"); $gullet->unread(Tokenize($2), T_CR);
    //           last; }
    //         push(@lines, $line . "\n"); }
    //       pop(@lines) if $lines[-1] eq "\n";
    //       // Note last line ends up as Whatsit's "trailer"
    //       if (my $b = LookupValue('@environment@verbatim@atend')) {
    //         push(@lines, ToString(Digest(@$b))); }
    //       $stomach->egroup;
    //       $whatsit->setBody(map { Box($_, $font, $loc, T_OTHER($_)) } @lines, $end);
    //       return; }],
    }),
    before_construct => construct!(document, whatsit, state, { document.maybe_close_element("ltx:p", state); })
  );

  // DefPrimitiveI('\@vobeyspaces', undef, sub {
  //     AssignCatcode(" " => 13);
  //     Let(T_ACTIVE(" "), '\nobreakspace');
  //     return });

  // WARNING: Need to be careful about what catcodes are active here
  DefMacro!("\\verb", sub[gullet, args, state] {
    let mouth = gullet.get_mouth_mut().unwrap();
    state.begin_semiverbatim(Some(vec!['%', '\\', '{', '}']));
    let mut init = mouth.read_token(state);
    if let Some(ref init_token) = init {
      if init_token.as_str() == "*" {
        init = mouth.read_token(state); // Should I bother handling \verb* ?
      }
    }
    if let Some(ref init_token) = init {
      let body = mouth.read_tokens(Some(init_token), state);
      state.end_semiverbatim();
      let cs = if state.lookup_bool("IN_MATH") { T_CS!("\\@math@verb") } else { T_CS!("\\@text@verb") };
      Ok(Invocation!(cs, vec![Tokens!(init.unwrap()), body], gullet, state)?)
    } else { // typically something read too far got \verb and the content is somewhere else..?
      error!(target: "expected:delimiter", "Verbatim argument lost\n Bindings for preceding code is probably broken");
      state.end_semiverbatim();
      Ok(Tokens!())
    }
  });

  DefConstructor!(
    "\\@text@verb{}{}",
    "<ltx:verbatim font='#font'>#2</ltx:verbatim>"
  );
  // TODO:
  // beforeDigest => [sub { $_[0]->bgroup; MergeFont(family => 'typewriter'); }],
  // afterDigest  => sub { $_[0]->egroup; },
  // # Since ltx:verbatim is both inline & block, we have to fudge inline mode
  // beforeConstruct => sub {
  //   $_[0]->canContain($_[0]->getElement, '#PCDATA')
  //     || $_[0]->openElement('ltx:p'); },
  // reversion => '\verb#1#2#1');
  DefConstructor!("\\@math@verb{}{}", "#2"); // Will already end up wrapped as XMTok!
                                             // TODO:
                                             // beforeDigest => [sub { $_[0]->bgroup; MergeFont(family => 'typewriter'); }],
                                             // afterDigest  => sub { $_[0]->egroup; },
                                             // reversion    => '\verb#1#2#1');

  // // Actually, latex sets catcode to 13 ... is this close enough?
  // DefPrimitiveI('\obeycr',    undef, sub { AssignValue('PRESERVE_NEWLINES' => 1); });
  // DefPrimitiveI('\restorecr', undef, sub { AssignValue('PRESERVE_NEWLINES' => 0); });

  // DefMacroI('\normalsfcodes', undef, Tokens());
  Ok(())
}
